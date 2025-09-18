// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

//! # Structural Code Replacement Engine
//!
//! Generates replacement code by traversing replacement templates and substituting
//! meta-variables with captured content from pattern matches.
//!
//! ## Core Concept
//!
//! Structural replacement uses AST-based templates rather than simple string substitution.
//! The replacement template is parsed into an AST, then meta-variables in that AST
//! are replaced with content captured during pattern matching.
//!
//! ## Process Overview
//!
//! 1. **Parse replacement template** - Convert replacement string to AST
//! 2. **Traverse AST nodes** - Visit each node in the replacement template
//! 3. **Identify meta-variables** - Find nodes that match meta-variable patterns
//! 4. **Substitute content** - Replace meta-variables with captured content
//! 5. **Generate output** - Combine unchanged and replaced content into final result
//!
//! ## Example
//!
//! **Pattern:** `function $NAME($$$PARAMS) { $$$BODY }`
//! **Replacement template:** `async function $NAME($$$PARAMS) { $$$BODY }`
//! **Captured variables:**
//! - `$NAME` → `"calculateSum"`
//! - `$$$PARAMS` → `"a, b"`
//! - `$$$BODY` → `"return a + b;"`
//!
//! **Result:** `async function calculateSum(a, b) { return a + b; }`
//!
//! ## Key Functions
//!
//! - [`gen_replacement`] - Main entry point for generating replacement content
//! - [`collect_edits`] - Traverse replacement template and collect substitution edits
//! - [`merge_edits_to_vec`] - Combine original content with edits to produce final result
//!
//! ## Algorithm Details
//!
//! Uses a post-order depth-first traversal to visit all nodes in the replacement
//! template. When a meta-variable is found, it's replaced with the corresponding
//! captured content. The traversal stops at nodes that match meta-variables to
//! avoid processing their children unnecessarily.
//!
//! ## Advantages
//!
//! - **Syntax-aware** - Respects language syntax and structure
//! - **Precise** - Only replaces intended meta-variables, not similar text
//! - **Efficient** - Single-pass traversal with minimal memory allocation
//! - **Language-agnostic** - Works with any language that has AST support

use super::{Edit, Underlying};
use crate::language::Language;
use crate::meta_var::MetaVarEnv;
use crate::source::{Content, SgNode};
use crate::{Doc, Node, NodeMatch, Root};

/// Generate replacement content by substituting meta-variables in a template AST.
///
/// Takes a replacement template (parsed as an AST) and substitutes any meta-variables
/// found in it with content captured during pattern matching.
///
/// # Parameters
///
/// - `root` - The replacement template parsed as an AST
/// - `nm` - Node match containing captured meta-variables
///
/// # Returns
///
/// Raw bytes representing the final replacement content
///
/// # Example
///
/// ```rust,ignore
/// // Template: "async function $NAME() { $$$BODY }"
/// // Variables: $NAME="test", $$$BODY="return 42;"
/// // Result: "async function test() { return 42; }"
/// let replacement = gen_replacement(&template_root, &node_match);
/// ```
pub fn gen_replacement<D: Doc>(root: &Root<D>, nm: &NodeMatch<D>) -> Underlying<D> {
    let edits = collect_edits(root, nm.get_env(), nm.lang());
    merge_edits_to_vec(edits, root)
}

/// Traverse the replacement template AST and collect edits for meta-variable substitution.
///
/// Performs a post-order depth-first traversal of the replacement template,
/// identifying nodes that represent meta-variables and creating edit operations
/// to replace them with captured content.
///
/// # Parameters
///
/// - `root` - Root of the replacement template AST
/// - `env` - Meta-variable environment with captured content
/// - `lang` - Language implementation for meta-variable extraction
///
/// # Returns
///
/// Vector of edit operations to apply for meta-variable substitution
fn collect_edits<D: Doc>(root: &Root<D>, env: &MetaVarEnv<D>, lang: &D::Lang) -> Vec<Edit<D>> {
    let mut node = root.root();
    let root_id = node.node_id();
    let mut edits = vec![];

    // this is a post-order DFS that stops traversal when the node matches
    'outer: loop {
        if let Some(text) = get_meta_var_replacement(&node, env, lang) {
            let range = node.range();
            let position = range.start;
            let length = range.len();
            edits.push(Edit::<D> {
                position,
                deleted_length: length,
                inserted_text: text,
            });
        } else if let Some(first_child) = node.child(0) {
            // traverse down to child
            node = first_child;
            continue;
        } else if node.inner.is_missing() {
            // TODO: better handling missing node
            if let Some(sibling) = node.next() {
                node = sibling;
                continue;
            }
            break;
        }
        // traverse up to parent until getting to root
        loop {
            // come back to the root node, terminating dfs
            if node.node_id() == root_id {
                break 'outer;
            }
            if let Some(sibling) = node.next() {
                node = sibling;
                break;
            }
            node = node.parent().unwrap();
        }
    }
    // add the missing one
    edits.push(Edit::<D> {
        position: root.root().range().end,
        deleted_length: 0,
        inserted_text: vec![],
    });
    edits
}

fn merge_edits_to_vec<D: Doc>(edits: Vec<Edit<D>>, root: &Root<D>) -> Underlying<D> {
    let mut ret = vec![];
    let mut start = 0;
    for edit in edits {
        debug_assert!(start <= edit.position, "Edit must be ordered!");
        ret.extend(
            root.doc
                .get_source()
                .get_range(start..edit.position)
                .iter()
                .cloned(),
        );
        ret.extend(edit.inserted_text.iter().cloned());
        start = edit.position + edit.deleted_length;
    }
    ret
}

fn get_meta_var_replacement<D: Doc>(
    node: &Node<D>,
    env: &MetaVarEnv<D>,
    lang: &D::Lang,
) -> Option<Underlying<D>> {
    if !node.is_named_leaf() {
        return None;
    }
    let meta_var = lang.extract_meta_var(&node.text())?;
    let replaced = env.get_var_bytes(&meta_var)?;
    Some(replaced.to_vec())
}

#[cfg(test)]
mod test {
    use crate::language::Tsx;
    use crate::meta_var::MetaVarEnv;
    use crate::{NodeMatch, Root, replacer::Replacer, tree_sitter::LanguageExt};
    use thread_utils::RapidMap;

    fn test_pattern_replace(replacer: &str, vars: &[(&str, &str)], expected: &str) {
        let mut env = MetaVarEnv::new();
        let roots: Vec<_> = vars.iter().map(|(v, p)| (v, Tsx.ast_grep(p))).collect();
        for (var, root) in &roots {
            env.insert(var, root.root());
        }
        let dummy = Tsx.ast_grep("dummy");
        let node_match = NodeMatch::new(dummy.root(), env.clone());
        let replacer = Root::str(replacer, Tsx);
        let replaced = replacer.generate_replacement(&node_match);
        let replaced = String::from_utf8_lossy(&replaced);
        assert_eq!(
            replaced,
            expected,
            "wrong replacement {replaced} {expected} {:?}",
            RapidMap::from(env)
        );
    }

    #[test]
    fn test_no_env() {
        test_pattern_replace("let a = 123", &[], "let a = 123");
        test_pattern_replace(
            "console.log('hello world'); let b = 123;",
            &[],
            "console.log('hello world'); let b = 123;",
        );
    }

    #[test]
    fn test_single_env() {
        test_pattern_replace("let a = $A", &[("A", "123")], "let a = 123");
        test_pattern_replace(
            "console.log($HW); let b = 123;",
            &[("HW", "'hello world'")],
            "console.log('hello world'); let b = 123;",
        );
    }

    #[test]
    fn test_multiple_env() {
        test_pattern_replace("let $V = $A", &[("A", "123"), ("V", "a")], "let a = 123");
        test_pattern_replace(
            "console.log($HW); let $B = 123;",
            &[("HW", "'hello world'"), ("B", "b")],
            "console.log('hello world'); let b = 123;",
        );
    }

    #[test]
    fn test_multiple_occurrences() {
        test_pattern_replace("let $A = $A", &[("A", "a")], "let a = a");
        test_pattern_replace("var $A = () => $A", &[("A", "a")], "var a = () => a");
        test_pattern_replace(
            "const $A = () => { console.log($B); $A(); };",
            &[("B", "'hello world'"), ("A", "a")],
            "const a = () => { console.log('hello world'); a(); };",
        );
    }

    fn test_ellipsis_replace(replacer: &str, vars: &[(&str, &str)], expected: &str) {
        let mut env = MetaVarEnv::new();
        let roots: Vec<_> = vars.iter().map(|(v, p)| (v, Tsx.ast_grep(p))).collect();
        for (var, root) in &roots {
            env.insert_multi(var, root.root().children().collect());
        }
        let dummy = Tsx.ast_grep("dummy");
        let node_match = NodeMatch::new(dummy.root(), env.clone());
        let replacer = Root::str(replacer, Tsx);
        let replaced = replacer.generate_replacement(&node_match);
        let replaced = String::from_utf8_lossy(&replaced);
        assert_eq!(
            replaced,
            expected,
            "wrong replacement {replaced} {expected} {:?}",
            RapidMap::from(env)
        );
    }

    #[test]
    fn test_ellipsis_meta_var() {
        test_ellipsis_replace(
            "let a = () => { $$$B }",
            &[("B", "alert('works!')")],
            "let a = () => { alert('works!') }",
        );
        test_ellipsis_replace(
            "let a = () => { $$$B }",
            &[("B", "alert('works!');console.log(123)")],
            "let a = () => { alert('works!');console.log(123) }",
        );
    }

    #[test]
    fn test_multi_ellipsis() {
        test_ellipsis_replace(
            "import {$$$A, B, $$$C} from 'a'",
            &[("A", "A"), ("C", "C")],
            "import {A, B, C} from 'a'",
        );
    }

    #[test]
    fn test_replace_in_string() {
        test_pattern_replace("'$A'", &[("A", "123")], "'123'");
    }

    #[test]
    fn test_nested_matching_replace() {
        // TODO
    }
}
