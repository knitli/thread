// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

use crate::{Rule, RuleCore};

use thread_ast_engine::meta_var::MetaVarEnv;
use thread_ast_engine::{Doc, Matcher, Node};

use bit_set::BitSet;
use thiserror::Error;

use std::borrow::Cow;
use std::sync::{Arc, RwLock, Weak};
use thread_utilities::{RapidMap, RapidSet, set_with_capacity};

#[derive(Debug)]
pub struct Registration<R>(Arc<RwLock<Arc<RapidMap<String, R>>>>);

impl<R> Clone for Registration<R> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<R> Registration<R> {
    fn read(&self) -> Arc<RapidMap<String, R>> {
        self.0.read().unwrap_or_else(|e| e.into_inner()).clone()
    }
    pub(crate) fn contains_key(&self, key: &str) -> bool {
        self.read().contains_key(key)
    }
    fn update<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut RapidMap<String, R>) -> T,
        R: Clone,
    {
        let mut lock = self.0.write().unwrap_or_else(|e| e.into_inner());
        let mut new_map = (**lock).clone();
        let ret = f(&mut new_map);
        *lock = Arc::new(new_map);
        ret
    }
}
pub type GlobalRules = Registration<RuleCore>;

impl GlobalRules {
    pub fn insert(&self, id: &str, rule: RuleCore) -> Result<(), ReferentRuleError> {
        self.update(|map| {
            if map.contains_key(id) {
                return Err(ReferentRuleError::DuplicateRule(id.into()));
            }
            map.insert(id.to_string(), rule);
            Ok(())
        })
    }
}

impl<R> Default for Registration<R> {
    fn default() -> Self {
        Self(Arc::new(RwLock::new(Arc::new(RapidMap::default()))))
    }
}

impl<R> From<Arc<RwLock<Arc<RapidMap<String, R>>>>> for Registration<R> {
    fn from(inner: Arc<RwLock<Arc<RapidMap<String, R>>>>) -> Self {
        Self(inner)
    }
}

#[derive(Clone, Debug, Default)]
pub struct RuleRegistration {
    /// utility rule to every RuleCore, every sub-rule has its own local utility
    local: Registration<Rule>,
    /// global rules are shared by all RuleConfigs. It is a singleton.
    global: Registration<RuleCore>,
    /// Every RuleConfig has its own rewriters. But sub-rules share parent's rewriters.
    rewriters: Registration<RuleCore>,
}

// these are shit code
impl RuleRegistration {
    pub fn get_rewriters(&self) -> Arc<RapidMap<String, RuleCore>> {
        self.rewriters.read()
    }

    pub fn has_global(&self, id: &str) -> bool {
        self.global.contains_key(id)
    }

    pub fn from_globals(global: &GlobalRules) -> Self {
        Self {
            local: Default::default(),
            global: global.clone(),
            rewriters: Default::default(),
        }
    }

    fn get_ref(&self) -> RegistrationRef {
        let local = Arc::downgrade(&self.local.0);
        let global = Arc::downgrade(&self.global.0);
        RegistrationRef { local, global }
    }

    pub(crate) fn insert_local(&self, id: &str, rule: Rule) -> Result<(), ReferentRuleError> {
        if rule.check_cyclic(id) {
            return Err(ReferentRuleError::CyclicRule(id.into()));
        }
        self.local.update(|map| {
            if map.contains_key(id) {
                return Err(ReferentRuleError::DuplicateRule(id.into()));
            }
            map.insert(id.to_string(), rule);
            Ok(())
        })
    }

    pub(crate) fn insert_rewriter(&self, id: &str, rewriter: RuleCore) {
        self.rewriters.insert(id, rewriter).expect("should work");
    }

    pub(crate) fn get_local_util_vars(&self) -> RapidSet<String> {
        let utils = self.local.read();
        let size = utils.len();
        if size == 0 {
            return RapidSet::default();
        }
        // this gets closer to the actual size
        let mut ret = set_with_capacity(size);
        for rule in utils.values() {
            for v in rule.defined_vars() {
                ret.insert(v);
            }
        }
        ret
    }
}

/// RegistrationRef must use Weak pointer to avoid
/// cyclic reference in RuleRegistration
#[derive(Clone, Debug)]
struct RegistrationRef {
    local: Weak<RwLock<Arc<RapidMap<String, Rule>>>>,
    global: Weak<RwLock<Arc<RapidMap<String, RuleCore>>>>,
}
impl RegistrationRef {
    fn get_local(&self) -> Arc<RapidMap<String, Rule>> {
        let lock = self
            .local
            .upgrade()
            .expect("Rule Registration must be kept alive");
        lock.read().unwrap_or_else(|e| e.into_inner()).clone()
    }
    fn get_global(&self) -> Arc<RapidMap<String, RuleCore>> {
        let lock = self
            .global
            .upgrade()
            .expect("Rule Registration must be kept alive");
        lock.read().unwrap_or_else(|e| e.into_inner()).clone()
    }
}

#[derive(Error, Debug)]
pub enum ReferentRuleError {
    #[error("Rule `{0}` is not defined.")]
    UndefinedUtil(String),
    #[error("Duplicate rule id `{0}` is found.")]
    DuplicateRule(String),
    #[error("Rule `{0}` has a cyclic dependency in its `matches` sub-rule.")]
    CyclicRule(String),
}

#[derive(Clone, Debug)]
pub struct ReferentRule {
    pub(crate) rule_id: String,
    reg_ref: RegistrationRef,
}

impl ReferentRule {
    pub fn try_new(
        rule_id: String,
        registration: &RuleRegistration,
    ) -> Result<Self, ReferentRuleError> {
        Ok(Self {
            reg_ref: registration.get_ref(),
            rule_id,
        })
    }

    fn eval_local<F, T>(&self, func: F) -> Option<T>
    where
        F: FnOnce(&Rule) -> T,
    {
        let rules = self.reg_ref.get_local();
        let rule = rules.get(&self.rule_id)?;
        Some(func(rule))
    }

    fn eval_global<F, T>(&self, func: F) -> Option<T>
    where
        F: FnOnce(&RuleCore) -> T,
    {
        let rules = self.reg_ref.get_global();
        let rule = rules.get(&self.rule_id)?;
        Some(func(rule))
    }

    pub(super) fn verify_util(&self) -> Result<(), ReferentRuleError> {
        let rules = self.reg_ref.get_local();
        if rules.contains_key(&self.rule_id) {
            return Ok(());
        }
        let rules = self.reg_ref.get_global();
        if rules.contains_key(&self.rule_id) {
            return Ok(());
        }
        Err(ReferentRuleError::UndefinedUtil(self.rule_id.clone()))
    }
}

impl Matcher for ReferentRule {
    fn match_node_with_env<'tree, D: Doc>(
        &self,
        node: Node<'tree, D>,
        env: &mut Cow<MetaVarEnv<'tree, D>>,
    ) -> Option<Node<'tree, D>> {
        self.eval_local(|r| r.match_node_with_env(node.clone(), env))
            .or_else(|| self.eval_global(|r| r.match_node_with_env(node, env)))
            .flatten()
    }
    fn potential_kinds(&self) -> Option<BitSet> {
        self.eval_local(|r| {
            debug_assert!(!r.check_cyclic(&self.rule_id), "no cyclic rule allowed");
            r.potential_kinds()
        })
        .or_else(|| {
            self.eval_global(|r| {
                debug_assert!(!r.check_cyclic(&self.rule_id), "no cyclic rule allowed");
                r.potential_kinds()
            })
        })
        .flatten()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rule::Rule;
    use crate::test::TypeScript as TS;
    use thread_ast_engine::Pattern;
    use thread_ast_engine::ops as o;

    type Result = std::result::Result<(), ReferentRuleError>;

    #[test]
    fn test_cyclic_error() -> Result {
        let registration = RuleRegistration::default();
        let rule = ReferentRule::try_new("test".into(), &registration)?;
        let rule = Rule::Matches(rule);
        let error = registration.insert_local("test", rule);
        assert!(matches!(error, Err(ReferentRuleError::CyclicRule(_))));
        Ok(())
    }

    #[test]
    fn test_cyclic_all() -> Result {
        let registration = RuleRegistration::default();
        let rule = ReferentRule::try_new("test".into(), &registration)?;
        let rule = Rule::All(o::All::new(std::iter::once(Rule::Matches(rule))));
        let error = registration.insert_local("test", rule);
        assert!(matches!(error, Err(ReferentRuleError::CyclicRule(_))));
        Ok(())
    }

    #[test]
    fn test_cyclic_not() -> Result {
        let registration = RuleRegistration::default();
        let rule = ReferentRule::try_new("test".into(), &registration)?;
        let rule = Rule::Not(Box::new(o::Not::new(Rule::Matches(rule))));
        let error = registration.insert_local("test", rule);
        assert!(matches!(error, Err(ReferentRuleError::CyclicRule(_))));
        Ok(())
    }

    #[test]
    fn test_success_rule() -> Result {
        let registration = RuleRegistration::default();
        let rule = ReferentRule::try_new("test".into(), &registration)?;
        let pattern = Rule::Pattern(Pattern::new("some", &TS::Tsx));
        let ret = registration.insert_local("test", pattern);
        assert!(ret.is_ok());
        assert!(rule.potential_kinds().is_some());
        Ok(())
    }

    #[test]
    fn test_has_global() -> Result {
        let globals = GlobalRules::default();
        let pattern = Rule::Pattern(Pattern::new("some", &TS::Tsx));
        let rule_core = crate::RuleCore::new(pattern);
        globals.insert("global_rule", rule_core)?;
        let registration = RuleRegistration::from_globals(&globals);
        assert!(registration.has_global("global_rule"));
        assert!(!registration.has_global("not_present"));
        Ok(())
    }
}
