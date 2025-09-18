// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

mod parse;
mod rewrite;
mod string_case;
mod trans;

use crate::{DeserializeEnv, RuleCore};

use thread_ast_engine::Doc;
use thread_ast_engine::Language;
use thread_ast_engine::meta_var::MetaVarEnv;
use thread_ast_engine::meta_var::MetaVariable;

use parse::ParseTransError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use thread_utils::RapidMap;

pub use trans::Trans;

#[derive(Serialize, Deserialize, Clone, JsonSchema, Debug)]
#[serde(untagged)]
pub enum Transformation {
    Simplied(String),
    Object(Trans<String>),
}

impl Transformation {
    pub fn parse<L: Language>(&self, lang: &L) -> Result<Trans<MetaVariable>, TransformError> {
        match self {
            Transformation::Simplied(s) => {
                let t: Trans<String> = s.parse()?;
                t.parse(lang)
            }
            Transformation::Object(t) => t.parse(lang),
        }
    }
}

#[derive(Error, Debug)]
pub enum TransformError {
    #[error("Cannot parse transform string.")]
    Parse(#[from] ParseTransError),
    #[error("`{0}` has a cyclic dependency.")]
    Cyclic(String),
    #[error("Transform var `{0}` has already defined.")]
    AlreadyDefined(String),
    #[error("source `{0}` should be $-prefixed.")]
    MalformedVar(String),
}

#[derive(Clone, Debug)]
pub struct Transform {
    transforms: Vec<(String, Trans<MetaVariable>)>,
}

impl Transform {
    pub fn deserialize<L: Language>(
        map: &RapidMap<String, Transformation>,
        env: &DeserializeEnv<L>,
    ) -> Result<Self, TransformError> {
        let map: Result<_, _> = map
            .iter()
            .map(|(key, val)| val.parse(&env.lang).map(|t| (key.to_string(), t)))
            .collect();
        let map = map?;
        let order = env
            .get_transform_order(&map)
            .map_err(TransformError::Cyclic)?;
        let transforms = order
            .iter()
            .map(|&key| (key.to_string(), map[key].clone()))
            .collect();
        Ok(Self { transforms })
    }

    pub fn apply_transform<'c, D: Doc>(
        &self,
        env: &mut MetaVarEnv<'c, D>,
        rewriters: &RapidMap<String, RuleCore>,
        enclosing_env: &MetaVarEnv<'c, D>,
    ) {
        let mut ctx = Ctx {
            env,
            rewriters,
            enclosing_env,
        };
        for (key, tr) in &self.transforms {
            tr.insert(key, &mut ctx);
        }
    }

    pub(crate) fn keys(&self) -> impl Iterator<Item = &String> {
        self.transforms.iter().map(|t| &t.0)
    }

    pub(crate) fn values(&self) -> impl Iterator<Item = &Trans<MetaVariable>> {
        self.transforms.iter().map(|t| &t.1)
    }
}

// two lifetime to represent env root lifetime and lang/trans lifetime
struct Ctx<'b, 'c, D: Doc> {
    rewriters: &'b RapidMap<String, RuleCore>,
    env: &'b mut MetaVarEnv<'c, D>,
    enclosing_env: &'b MetaVarEnv<'c, D>,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::from_str;
    use crate::test::TypeScript;
    use thread_ast_engine::tree_sitter::LanguageExt;

    #[test]
    fn test_transform_str() {}

    #[test]
    fn test_single_cyclic_transform() {
        let mut trans = RapidMap::default();
        let trans_a = from_str("substring: {source: $A}").unwrap();
        trans.insert("A".into(), trans_a);
        let env = DeserializeEnv::new(TypeScript::Tsx);
        match Transform::deserialize(&trans, &env) {
            Err(TransformError::Cyclic(a)) => assert_eq!(a, "A"),
            _ => panic!("unexpected error"),
        }
    }

    #[test]
    fn test_cyclic_transform() {
        let mut trans = RapidMap::default();
        let trans_a = from_str("substring: {source: $B}").unwrap();
        trans.insert("A".into(), trans_a);
        let trans_b = from_str("substring: {source: $A}").unwrap();
        trans.insert("B".into(), trans_b);
        let env = DeserializeEnv::new(TypeScript::Tsx);
        let ret = Transform::deserialize(&trans, &env);
        assert!(matches!(ret, Err(TransformError::Cyclic(_))));
    }

    #[test]
    fn test_transform_use_matched() {
        let mut trans = RapidMap::default();
        let trans_a = from_str("substring: {source: $C}").unwrap();
        trans.insert("A".into(), trans_a);
        let trans_b = from_str("substring: {source: $A}").unwrap();
        trans.insert("B".into(), trans_b);
        let env = DeserializeEnv::new(TypeScript::Tsx);
        let ret = Transform::deserialize(&trans, &env);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_transform_indentation() {
        let src = "
if (true) {
  let a = {
    b: 123
  }
}
";
        let expected = "{
  b: 123
}";
        let mut trans = RapidMap::default();
        let tr = from_str("{ substring: { source: $A } }").expect("should work");
        trans.insert("TR".into(), tr);
        let grep = TypeScript::Tsx.ast_grep(src);
        let root = grep.root();
        let mut nm = root.find("let a = $A").expect("should find");
        let env = DeserializeEnv::new(TypeScript::Tsx);
        let trans = Transform::deserialize(&trans, &env).expect("should deserialize");
        trans.apply_transform(nm.get_env_mut(), &Default::default(), &Default::default());
        let actual = nm.get_env().get_transformed("TR").expect("should have TR");
        let actual = std::str::from_utf8(actual).expect("should work");
        assert_eq!(actual, expected);
    }
}
