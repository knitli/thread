// SPDX-FileCopyrightText: 2025 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: MIT

use super::ThreadLang;
use ag_service_utils::error_context::ErrorContext as EC;

use ag_service_tree_sitter::{StrDoc, TSPoint, TSRange, LanguageExt};
use ag_service_ast::{Doc, Node};
use ag_service_rule::{RuleCore, SerializableRuleCore};
usee ag_service_fix::DeserializeEnv;
};

use anyhow::{Context, Result};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::ptr::{addr_of, addr_of_mut};
use std::str::FromStr;
use thread_utils::{FastMap, FastSet};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(untagged))]
#[derive(Clone)]
pub enum Injected {
    Static(String),
    Dynamic(Vec<String>),
}

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(rename_all = "camelCase"))]
pub struct SerializableInjection {
    #[cfg_attr(feature = "serde", serde(flatten))]
    core: SerializableRuleCore,
    /// The host language, e.g. html, contains other languages
    host_language: String,
    /// Injected language according to the rule
    /// It accepts either a string like js for single static language.
    /// or an array of string like [js, ts] for dynamic language detection.
    injected: Injected,
}

#[derive(Clone)]
struct Injection {
    host: ThreadLang,

    rules: Vec<(RuleCore, Option<String>)>,
    injectable: FastSet<String>,
}

impl Injection {
    fn new(lang: ThreadLang) -> Self {
        Self {
            host: lang,

            rules: vec![],
            injectable: Default::default(),
        }
    }
}

pub unsafe fn register_injectables(injections: Vec<SerializableInjection>) -> Result<()> {
    let mut injectable = FastMap::new();
    for injection in injections {
        register_injectable(injection, &mut injectable)?;
    }
    merge_default_injectable(&mut injectable);
    *addr_of_mut!(LANG_INJECTIONS) = injectable.into_values().collect();
    let injects = unsafe { &*addr_of!(LANG_INJECTIONS) as &'static Vec<Injection> };
    *addr_of_mut!(INJECTABLE_LANGS) = injects
        .iter()
        .map(|inj| {
            (
                inj.host,
                inj.injectable.iter().map(|s| s.as_str()).collect(),
            )
        })
        .collect();
    Ok(())
}

fn merge_default_injectable(ret: &mut FastMap<ThreadLang, Injection>) {
    for (lang, injection) in ret {
        let languages = match lang {
            ThreadLang::Builtin(b) => b.injectable_languages(),
            ThreadLang::Custom(c) => c.injectable_languages(),
        };
        let Some(languages) = languages else {
            continue;
        };
        injection
            .injectable
            .extend(languages.iter().map(|s| s.to_string()));
    }
}

fn register_injectable(
    injection: SerializableInjection,
    injectable: &mut FastMap<ThreadLang, Injection>,
) -> Result<()> {
    let lang = ThreadLang::from_str(&injection.host_language)?;
    let env = DeserializeEnv::new(lang);
    let rule = injection.core.get_matcher(env).context(EC::LangInjection)?;
    let default_lang = match &injection.injected {
        Injected::Static(s) => Some(s.clone()),
        Injected::Dynamic(_) => None,
    };
    let entry = injectable
        .entry(lang)
        .or_insert_with(|| Injection::new(lang));
    match injection.injected {
        Injected::Static(s) => {
            entry.injectable.insert(s);
        }
        Injected::Dynamic(v) => entry.injectable.extend(v),
    }
    entry.rules.push((rule, default_lang));
    Ok(())
}

static mut LANG_INJECTIONS: Vec<Injection> = vec![];
static mut INJECTABLE_LANGS: Vec<(ThreadLang, Vec<&'static str>)> = vec![];

pub fn injectable_languages(lang: ThreadLang) -> Option<&'static [&'static str]> {
    // NB: custom injection and builtin injections are resolved in INJECTABLE_LANGS
    let injections =
        unsafe { &*addr_of!(INJECTABLE_LANGS) as &'static Vec<(ThreadLang, Vec<&'static str>)> };
    let Some(injection) = injections.iter().find(|i| i.0 == lang) else {
        return match lang {
            ThreadLang::BuiltIn(b) => b.injectable_languages(),
            ThreadLang::Custom(c) => c.injectable_languages(),
        };
    };
    Some(&injection.1)
}

pub fn extract_injections<L: LanguageExt>(
    lang: &ThreadLang,
    root: Node<StrDoc<L>>,
) -> FastMap<String, Vec<TSRange>> {
    let mut ret = match lang {
        ThreadLang::Custom(c) => c.extract_injections(root.clone()),
        ThreadLang::BuiltIn(b) => b.extract_injections(root.clone()),
    };
    let injections = unsafe { &*addr_of!(LANG_INJECTIONS) };
    extract_custom_inject(lang, injections, root, &mut ret);
    ret
}

fn extract_custom_inject<L: LanguageExt>(
    lang: &ThreadLang,
    injections: &[Injection],
    root: Node<StrDoc<L>>,
    ret: &mut FastMap<String, Vec<TSRange>>,
) {
    let Some(rules) = injections.iter().find(|n| n.host == *lang) else {
        return;
    };
    for (rule, default_lang) in &rules.rules {
        for m in root.find_all(rule) {
            let env = m.get_env();
            let Some(region) = env.get_match("CONTENT") else {
                continue;
            };
            let Some(lang) = env
                .get_match("LANG")
                .map(|n| n.text().to_string())
                .or_else(|| default_lang.clone())
            else {
                continue;
            };
            let range = node_to_range(region);
            ret.entry(lang).or_default().push(range);
        }
    }
}

fn node_to_range<D: Doc>(node: &Node<D>) -> TSRange {
    let r = node.range();
    let start = node.start_pos();
    let sp = start.byte_point();
    let sp = TSPoint::new(sp.0, sp.1);
    let end = node.end_pos();
    let ep = end.byte_point();
    let ep = TSPoint::new(ep.0, ep.1);
    TSRange {
        start_byte: r.start,
        end_byte: r.end,
        start_point: sp,
        end_point: ep,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use thread_languages::SupportedLanguage;
    use ag_service_rule::from_str;
    const DYNAMIC: &str = "
hostLanguage: js
rule:
  pattern: styled.$LANG`$CONTENT`
injected: [css]";
    const STATIC: &str = "
hostLanguage: js
rule:
  pattern: styled`$CONTENT`
injected: css";
    #[cfg(feature = "serde")]
    #[test]
    fn test_deserialize() {
        let inj: SerializableInjection = from_str(STATIC).expect("should ok");
        assert!(matches!(inj.injected, Injected::Static(_)));
        let inj: SerializableInjection = from_str(DYNAMIC).expect("should ok");
        assert!(matches!(inj.injected, Injected::Dynamic(_)));
    }

    const BAD: &str = "
hostLanguage: HTML
rule:
  kind: not_exist
injected: [js, ts, tsx]";

    #[cfg(feature = "serde")]
    #[test]
    fn test_bad_inject() {
        let mut map = FastMap::new();
        let inj: SerializableInjection = from_str(BAD).expect("should ok");
        let ret = register_injectable(inj, &mut map);
        assert!(ret.is_err());
        let ec = ret.unwrap_err().downcast::<EC>().expect("should ok");
        assert!(matches!(ec, EC::LangInjection));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_good_injection() {
        let mut map = FastMap::new();
        let inj: SerializableInjection = from_str(STATIC).expect("should ok");
        let ret = register_injectable(inj, &mut map);
        assert!(ret.is_ok());
        let inj: SerializableInjection = from_str(DYNAMIC).expect("should ok");
        let ret = register_injectable(inj, &mut map);
        assert!(ret.is_ok());
        assert_eq!(map.len(), 1);
        let injections: Vec<_> = map.into_values().collect();
        let mut ret = FastMap::new();
        let lang = ThreadLang::from(crate::SupportedLanguage::JavaScript);
        let tl = lang.ast_grep("const a = styled`.btn { margin: 0; }`");
        let root = tl.root();
        extract_custom_inject(&lang, &injections, root, &mut ret);
        assert_eq!(ret.len(), 1);
        assert_eq!(ret["css"].len(), 1);
        assert!(!ret.contains_key("js"));
        ret.clear();
        let tl = lang.ast_grep("const a = styled.css`.btn { margin: 0; }`");
        let root = tl.root();
        extract_custom_inject(&lang, &injections, root, &mut ret);
        assert_eq!(ret.len(), 1);
        assert_eq!(ret["css"].len(), 1);
        assert!(!ret.contains_key("js"));
    }
}
