use ag_service_ast::{Doc, Node};
use ag_service_pattern::NodeMatch;
use ag_service_types::{Label, LabelConfig, LabelStyle};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::ops::Range;

use thread_utils::FastMap;

impl<'t, D: Doc> Label<'_, 't, D> {
    fn primary(n: &Node<'t, D>) -> Self {
        Self {
            style: LabelStyle::Primary,
            start_node: n.clone(),
            end_node: n.clone(),
            message: None,
        }
    }
    fn secondary(n: &Node<'t, D>) -> Self {
        Self {
            style: LabelStyle::Secondary,
            start_node: n.clone(),
            end_node: n.clone(),
            message: None,
        }
    }

    pub fn range(&self) -> Range<usize> {
        let start = self.start_node.range().start;
        let end = self.end_node.range().end;
        start..end
    }
}

pub fn get_labels_from_config<'r, 't, D: Doc>(
    config: &'r FastMap<String, LabelConfig>,
    node_match: &NodeMatch<'t, D>,
) -> Vec<Label<'r, 't, D>> {
    let env = node_match.get_env();
    config
        .iter()
        .filter_map(|(var, conf)| {
            let (start, end) = if let Some(n) = env.get_match(var) {
                (n.clone(), n.clone())
            } else {
                let ns = env.get_multiple_matches(var);
                let start = ns.first()?.clone();
                let end = ns.last()?.clone();
                (start, end)
            };
            Some(Label {
                style: conf.style.clone(),
                message: conf.message.as_deref(),
                start_node: start,
                end_node: end,
            })
        })
        .collect()
}

pub fn get_default_labels<'t, D: Doc>(n: &NodeMatch<'t, D>) -> Vec<Label<'static, 't, D>> {
    let mut ret = vec![Label::primary(n)];
    if let Some(secondary) = n.get_env().get_labels("secondary") {
        ret.extend(secondary.iter().map(Label::secondary));
    }
    ret
}

pub use {Label, LabelStyle, LabelConfig};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TypeScript;
    use ag_service_pattern::Pattern;
    use ag_service_tree_sitter::{LanguageExt, StrDoc};

    #[test]
    fn test_label_primary_secondary() {
        let doc = TypeScript::Tsx.ast_grep("let a = 1;");
        let root = doc.root();
        let label = Label::primary(&root);
        assert_eq!(label.style, LabelStyle::Primary);
        assert_eq!(label.range(), root.range());
        let label2 = Label::<'_, '_, StrDoc<TypeScript>>::secondary(&root);
        assert_eq!(label2.style, LabelStyle::Secondary);
    }

    #[test]
    fn test_get_labels_from_config_single() {
        let doc = TypeScript::Tsx.ast_grep("let foo = 42;");
        let pattern = Pattern::try_new("let $A = $B;", TypeScript::Tsx).unwrap();
        let m = doc.root().find(pattern).unwrap();
        let mut config = thread_utils::FastMap::new();
        config.insert(
            "A".to_string(),
            LabelConfig {
                style: LabelStyle::Primary,
                message: Some("var label".to_string()),
            },
        );
        let labels = get_labels_from_config(&config, &m);
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].style, LabelStyle::Primary);
        assert_eq!(labels[0].message, Some("var label"));
    }

    #[test]
    fn test_get_labels_from_config_multiple() {
        let doc = TypeScript::Tsx.ast_grep("let foo = 42, bar = 99;");
        let pattern = Pattern::try_new("let $A = $B, $C = $D;", TypeScript::Tsx).unwrap();
        let m = doc.root().find(pattern).unwrap();
        let mut config = thread_utils::FastMap::new();
        config.insert(
            "A".to_string(),
            LabelConfig {
                style: LabelStyle::Secondary,
                message: None,
            },
        );
        let labels = get_labels_from_config(&config, &m);
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].style, LabelStyle::Secondary);
    }

    #[test]
    fn test_get_default_labels() {
        let doc = TypeScript::Tsx.ast_grep("let foo = 42;");
        let pattern = Pattern::try_new("let $A = $B;", TypeScript::Tsx).unwrap();
        let m = doc.root().find(pattern).unwrap();
        let labels = get_default_labels(&m);
        assert!(!labels.is_empty());
        assert_eq!(labels[0].style, LabelStyle::Primary);
    }
}
