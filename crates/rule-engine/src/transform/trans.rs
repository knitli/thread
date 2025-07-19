// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

use super::rewrite::Rewrite;
use super::{Ctx, TransformError, string_case};
use thread_ast_engine::meta_var::MetaVariable;
use thread_ast_engine::source::Content;
use thread_ast_engine::{Doc, Language};
use thread_utils::is_ascii_simd;

use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::{OnceLock};

use string_case::{Separator, StringCase};

#[inline]
fn get_text_from_env<D: Doc>(var: &MetaVariable, ctx: &mut Ctx<'_, '_, D>) -> Option<String> {
    // TODO: check if topological sort has resolved transform dependency
    let bytes = ctx.env.get_var_bytes(var)?;
    Some(<D::Source as Content>::encode_bytes(bytes).into_owned())
}


/// Extracts a substring from the meta variable's text content.
///
/// Both `start_char` and `end_char` support negative indexing,
/// which counts character from the end of an array, moving backwards.
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Substring<T> {
    /// source meta variable to be transformed
    pub source: T,
    /// optional starting character index of the substring, defaults to 0.
    pub start_char: Option<i32>,
    /// optional ending character index of the substring, defaults to the end of the string.
    pub end_char: Option<i32>,
}

impl Substring<MetaVariable> {
    /// Computes the substring based on the provided character indices.
    #[inline]
    fn compute<D: Doc>(&self, ctx: &mut Ctx<'_, '_, D>) -> Option<String> {
        let text = get_text_from_env(&self.source, ctx)?;

        // SIMD-optimized ASCII check for better performance on longer strings
        if is_ascii_simd(&text) {
            return self.compute_ascii(&text);
        }
        // Fallback to standard ASCII check
        if text.is_ascii() {
            return self.compute_ascii(&text);
        }

        // UTF-8 path using char indices for boundary-safe slicing
        self.compute_unicode(&text)
    }

    /// Optimized substring for ASCII strings
    #[inline]
    fn compute_ascii(&self, text: &str) -> Option<String> {
        let len = text.len() as i32;
        let start = resolve_char(&self.start_char, 0, len) as usize;
        let end = resolve_char(&self.end_char, len, len) as usize;

        if start > end || start >= text.len() || end > text.len() {
            return Some(String::new());
        }

        Some(text[start..end].to_string())
    }

    /// UTF-8 aware substring using char boundaries
    #[inline]
    fn compute_unicode(&self, text: &str) -> Option<String> {
        let char_count = text.chars().count() as i32;
        let start_idx = resolve_char(&self.start_char, 0, char_count);
        let end_idx = resolve_char(&self.end_char, char_count, char_count);

        if start_idx > end_idx || start_idx >= char_count as usize {
            return Some(String::new());
        }

        // Use char_indices for efficient boundary detection
        let mut char_indices = text.char_indices();
        let start_byte = char_indices.nth(start_idx).map(|(i, _)| i).unwrap_or(text.len());

        if end_idx >= char_count as usize {
            return Some(text[start_byte..].to_string());
        }

        let end_byte = char_indices.nth(end_idx - start_idx - 1)
            .map(|(i, _)| i)
            .unwrap_or(text.len());

        Some(text[start_byte..end_byte].to_string())
    }
}

/// resolve relative negative char index to absolute index
/// e.g. -1 => len - 1, n > len => n
#[inline]
fn resolve_char(opt: &Option<i32>, dft: i32, len: i32) -> usize {
    let c = *opt.as_ref().unwrap_or(&dft);
    if c >= len {
        len as usize
    } else if c >= 0 {
        c as usize
    } else if len + c < 0 {
        0
    } else {
        debug_assert!(c < 0);
        (len + c) as usize
    }
}

/// Replaces a substring in the meta variable's text content with another string.
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Replace<T> {
    /// source meta variable to be transformed
    pub source: T,
    /// a regex to find substring to be replaced
    pub replace: String,
    /// the replacement string
    pub by: String,
    /// Cached compiled regex for performance
    #[serde(skip)]
    #[schemars(skip)]
    compiled_regex: OnceLock<Result<Regex, String>>,
}

impl<T> Replace<T> {
    /// Create a new Replace with empty cache
    #[inline]
    pub fn new(source: T, replace: String, by: String) -> Self {
        Self {
            source,
            replace,
            by,
            compiled_regex: OnceLock::new(),
        }
    }

    /// Get the cached compiled regex, compiling it if necessary
    #[inline]
    fn get_regex(&self) -> Option<&Regex> {
        let result = self.compiled_regex.get_or_init(|| {
            Regex::new(&self.replace).map_err(|e| e.to_string())
        });
        result.as_ref().ok()
    }
}

impl Replace<MetaVariable> {
    /// Computes the replacement of the matched text.
    #[inline]
    fn compute<D: Doc>(&self, ctx: &mut Ctx<'_, '_, D>) -> Option<String> {
        let text = get_text_from_env(&self.source, ctx)?;
        let re = self.get_regex()?;
        Some(re.replace_all(&text, &self.by).into_owned())
    }
}

/// Converts the source meta variable's text content to a specified case format.
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Convert<T> {
    /// source meta variable to be transformed
    pub source: T,
    /// the target case format to convert the text content to
    pub to_case: StringCase,
    /// optional separators to specify how to separate word
    pub separated_by: Option<Vec<Separator>>,
}

impl Convert<MetaVariable> {
    #[inline]
    fn compute<D: Doc>(&self, ctx: &mut Ctx<'_, '_, D>) -> Option<String> {
        let text = get_text_from_env(&self.source, ctx)?;
        Some(self.to_case.apply(&text, self.separated_by.as_deref()))
    }
}

/// Represents a transformation that can be applied to a matched AST node.
/// Available transformations are `substring`, `replace` and `convert`.
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Trans<T> {
    Substring(Substring<T>),
    Replace(Replace<T>),
    Convert(Convert<T>),
    Rewrite(Rewrite<T>),
}

impl<T> Trans<T> {
    fn source(&self) -> &T {
        use Trans as T;
        match self {
            T::Replace(r) => &r.source,
            T::Substring(s) => &s.source,
            T::Convert(c) => &c.source,
            T::Rewrite(r) => &r.source,
        }
    }
}

pub(crate) fn parse_meta_var<L: Language>(
    src: &str,
    lang: &L,
) -> Result<MetaVariable, TransformError> {
    let source = lang.pre_process_pattern(src);
    if let Some(var) = lang.extract_meta_var(&source) {
        Ok(var)
    } else {
        Err(TransformError::MalformedVar(src.to_string()))
    }
}

impl Trans<String> {
    pub fn parse<L: Language>(&self, lang: &L) -> Result<Trans<MetaVariable>, TransformError> {
        use Trans as T;
        Ok(match self {
            T::Replace(r) => T::Replace(Replace::new(
                parse_meta_var(&r.source, lang)?,
                r.replace.clone(),
                r.by.clone(),
            )),
            T::Substring(s) => T::Substring(Substring {
                source: parse_meta_var(&s.source, lang)?,
                start_char: s.start_char,
                end_char: s.end_char,
            }),
            T::Convert(c) => T::Convert(Convert {
                source: parse_meta_var(&c.source, lang)?,
                to_case: c.to_case,
                separated_by: c.separated_by.clone(),
            }),
            T::Rewrite(r) => T::Rewrite(r.parse(lang)?),
        })
    }
}
impl Trans<MetaVariable> {
    pub(super) fn insert<D: Doc>(&self, key: &str, ctx: &mut Ctx<'_, '_, D>) {
        let src = self.source();
        // TODO: add this debug assertion back
        // debug_assert!(ctx.env.get_transformed(key).is_none());
        // avoid cyclic
        ctx.env.insert_transformation(src, key, vec![]);
        let opt = self.compute(ctx);
        let bytes = if let Some(s) = opt {
            <D::Source as Content>::decode_str(&s).to_vec()
        } else {
            vec![]
        };
        ctx.env.insert_transformation(src, key, bytes);
    }
    fn compute<D: Doc>(&self, ctx: &mut Ctx<'_, '_, D>) -> Option<String> {
        use Trans as T;
        match self {
            T::Replace(r) => r.compute(ctx),
            T::Substring(s) => s.compute(ctx),
            T::Convert(c) => c.compute(ctx),
            T::Rewrite(r) => r.compute(ctx),
        }
    }

    pub fn used_rewriters(&self) -> &[String] {
        use Trans as T;
        match self {
            T::Replace(_) => &[],
            T::Substring(_) => &[],
            T::Convert(_) => &[],
            T::Rewrite(r) => &r.rewriters,
        }
    }
    pub fn used_vars(&self) -> &str {
        let s = self.source();
        use MetaVariable as MV;
        match s {
            MV::Capture(v, _) => v,
            MV::MultiCapture(v) => v,
            MV::Dropped(_) | MV::Multiple => panic!("transform var must be named"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::Transform;
    use super::*;
    use crate::test::TypeScript;
    use crate::{DeserializeEnv, Transformation};
    use serde_yaml::with::singleton_map_recursive;
    use thread_ast_engine::tree_sitter::LanguageExt;
    use thread_utils::RapidMap;

    type R = std::result::Result<(), ()>;

    fn get_transformed(src: &str, pat: &str, trans: &Trans<String>) -> Option<String> {
        let grep = TypeScript::Tsx.ast_grep(src);
        let root = grep.root();
        let mut nm = root.find(pat).expect("should find");
        let mut ctx = Ctx {
            env: nm.get_env_mut(),
            rewriters: &Default::default(),
            enclosing_env: &Default::default(),
        };
        trans.parse(&TypeScript::Tsx).ok()?.compute(&mut ctx)
    }

    fn parse(trans: &str) -> Result<Trans<String>, ()> {
        let deserializer = serde_yaml::Deserializer::from_str(trans);
        singleton_map_recursive::deserialize(deserializer).map_err(|_| ())
    }

    #[test]
    fn test_transform_parse_error() {
        let str_trans = parse(r#"substring: { source: WRONG }"#).expect("should work");
        match str_trans.parse(&TypeScript::Tsx) {
            Err(TransformError::MalformedVar(n)) => assert_eq!(n, "WRONG"),
            _ => panic!("should be malformed var"),
        }
    }

    #[test]
    fn test_simple_replace() -> R {
        let trans = parse(
            r#"
      substring:
        source: "$A"
        startChar: 1
        endChar: -1
    "#,
        )?;
        let actual = get_transformed("let a = 123", "let a= $A", &trans).ok_or(())?;
        assert_eq!(actual, "2");
        Ok(())
    }

    #[test]
    fn test_no_end_char() -> R {
        let trans = parse(
            r#"
      substring:
        source: "$A"
        startChar: 1
    "#,
        )?;
        let actual = get_transformed("let a = 123", "let a= $A", &trans).ok_or(())?;
        assert_eq!(actual, "23");
        Ok(())
    }
    #[test]
    fn test_no_start_char() -> R {
        let trans = parse(
            r#"
      substring:
        source: "$A"
        endChar: -1
    "#,
        )?;
        let actual = get_transformed("let a = 123", "let a= $A", &trans).ok_or(())?;
        assert_eq!(actual, "12");
        Ok(())
    }

    #[test]
    fn test_replace() -> R {
        let trans = parse(
            r#"
      replace:
        source: "$A"
        replace: \d
        by: "b"
    "#,
        )?;
        let actual = get_transformed("let a = 123", "let a= $A", &trans).ok_or(())?;
        assert_eq!(actual, "bbb");
        Ok(())
    }

    #[test]
    fn test_wrong_rule() {
        let parsed = parse(
            r#"
      replace:
        source: "$A"
    "#,
        );
        assert!(parsed.is_err());
    }

    fn transform_env(trans: RapidMap<String, Trans<String>>) -> RapidMap<String, String> {
        let grep = TypeScript::Tsx.ast_grep("let a = 123");
        let root = grep.root();
        let trans = trans
            .into_iter()
            .map(|(k, v)| (k, Transformation::Object(v)))
            .collect();
        let mut nm = root.find("let a = $A").expect("should find");
        let env = DeserializeEnv::new(TypeScript::Tsx);
        let trans = Transform::deserialize(&trans, &env).expect("should deserialize");
        trans.apply_transform(nm.get_env_mut(), &Default::default(), &Default::default());
        nm.get_env().clone().into()
    }

    #[test]
    fn test_insert_env() -> R {
        let tr1 = parse(
            r#"
      replace:
        source: "$A"
        replace: \d
        by: "b"
    "#,
        )?;
        let tr2 = parse(
            r#"
      substring:
        source: "$A"
        startChar: 1
        endChar: -1
    "#,
        )?;
        let mut map = RapidMap::default();
        map.insert("TR1".into(), tr1);
        map.insert("TR2".into(), tr2);
        let env = transform_env(map);
        assert_eq!(env["TR1"], "bbb");
        assert_eq!(env["TR2"], "2");
        Ok(())
    }

    #[test]
    fn test_dependent_trans() -> R {
        let rep = parse(
            r#"
      replace:
        source: "$A"
        replace: \d
        by: "b"
    "#,
        )?;
        let sub = parse(
            r#"
      substring:
        source: "$REP"
        startChar: 1
        endChar: -1
    "#,
        )?;
        let up = parse(
            r#"
      convert:
        source: "$SUB"
        toCase: upperCase
    "#,
        )?;
        let mut map = RapidMap::default();
        map.insert("REP".into(), rep);
        map.insert("SUB".into(), sub);
        map.insert("UP".into(), up);
        let env = transform_env(map);
        assert_eq!(env["REP"], "bbb");
        assert_eq!(env["SUB"], "b");
        assert_eq!(env["UP"], "B");
        Ok(())
    }

    #[test]
    fn test_uppercase_convert() -> R {
        let trans = parse(
            r#"
      convert:
        source: "$A"
        toCase: upperCase
    "#,
        )?;
        let actual = get_transformed("let a = real_quiet_now", "let a = $A", &trans).ok_or(())?;
        assert_eq!(actual, "REAL_QUIET_NOW");
        Ok(())
    }

    #[test]
    fn test_capitalize_convert() -> R {
        let trans = parse(
            r#"
      convert:
        source: "$A"
        toCase: capitalize
    "#,
        )?;
        let actual = get_transformed("let a = snugglebunny", "let a = $A", &trans).ok_or(())?;
        assert_eq!(actual, "Snugglebunny");
        Ok(())
    }

    #[test]
    fn test_lowercase_convert() -> R {
        let trans = parse(
            r#"
      convert:
        source: "$A"
        toCase: lowerCase
    "#,
        )?;
        let actual = get_transformed("let a = SCREAMS", "let a = $A", &trans).ok_or(())?;
        assert_eq!(actual, "screams");
        Ok(())
    }

    #[test]
    fn test_separation_convert() -> R {
        let trans = parse(
            r#"
      convert:
        source: "$A"
        toCase: snakeCase
        separatedBy: [underscore]
    "#,
        )?;
        let actual = get_transformed("let a = camelCase_Not", "let a = $A", &trans).ok_or(())?;
        assert_eq!(actual, "camelcase_not");
        Ok(())
    }

    #[test]
    fn test_transform_indentation_with_insertion() -> R {
        let src = "
if (true) {
  let a = {
    b: 123
  }
}
";
        // note the indentation
        let expected = "{
    b: 123
  }";
        let tr = parse("{ substring: { source: $A } }")?;
        let actual = get_transformed(src, "let a = $A", &tr).ok_or(())?;
        assert_eq!(actual, expected);
        Ok(())
    }

    // TODO: add a symbolic test for Rewrite
}
