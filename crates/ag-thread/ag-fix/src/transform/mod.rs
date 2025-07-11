mod parse;
mod rewrite;
mod string_case;
mod trans;

use arg_service_rule::{DeserializeEnv, RuleCore};
use arg_service_ast::{MetaVarEnv, MetaVariable, Doc, Language};
use arg_service_types::{Transformation, Transform, TransformError};

use parse::parser::ParseTransError;
pub use parse:parser::Trans;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thread_utils::FastMap;
use thiserror::Error;


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

impl Transform {
    pub fn deserialize<L: Language>(
        map: &FastMap<String, Transformation>,
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
        rewriters: &FastMap<String, RuleCore>,
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

pub mod transformation {
    pub use super::Transformation;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::from_str;
    use crate::test::TypeScript;
    use ag_service_core::tree_sitter::LanguageExt;

    #[test]
    fn test_transform_str() {}

    #[test]
    fn test_single_cyclic_transform() {
        let mut trans = FastMap::new();
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
        let mut trans = FastMap::new();
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
        let mut trans = FastMap::new();
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
        let mut trans = FastMap::new();
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
