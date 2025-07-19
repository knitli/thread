// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

//! # Code Replacement and Transformation
//!
//! Tools for replacing and transforming matched AST nodes with new content.
//!
//! ## Core Concepts
//!
//! - [`Replacer`] - Trait for generating replacement content from matched nodes
//! - Template-based replacement using meta-variables (e.g., `"let $VAR = $VALUE"`)
//! - Structural replacement using other AST nodes
//! - Automatic indentation handling to preserve code formatting
//!
//! ## Built-in Replacers
//!
//! Several types implement [`Replacer`] out of the box:
//!
//! - **`&str`** - Template strings with meta-variable substitution
//! - **[`Root`]** - Replace with entire AST trees
//! - **[`Node`]** - Replace with specific nodes
//!
//! ## Examples
//!
//! ### Template Replacement
//!
//! ```rust,no_run
//! # use thread_ast_engine::Language;
//! # use thread_ast_engine::tree_sitter::LanguageExt;
//! # use thread_ast_engine::matcher::MatcherExt;
//! let mut ast = Language::Tsx.ast_grep("var x = 42;");
//!
//! // Replace using a template string
//! ast.replace("var $NAME = $VALUE", "const $NAME = $VALUE");
//! println!("{}", ast.generate()); // "const x = 42;"
//! ```
//!
//! ### Structural Replacement
//!
//! ```rust,no_run
//! # use thread_ast_engine::Language;
//! # use thread_ast_engine::tree_sitter::LanguageExt;
//! # use thread_ast_engine::matcher::MatcherExt;
//! let mut target = Language::Tsx.ast_grep("old_function();");
//! let replacement = Language::Tsx.ast_grep("new_function(42)");
//!
//! // Replace with another AST
//! target.replace("old_function()", replacement);
//! println!("{}", target.generate()); // "new_function(42);"
//! ```

use crate::matcher::Matcher;
use crate::meta_var::{MetaVariableID, Underlying, is_valid_meta_var_char};
use crate::{Doc, Node, NodeMatch, Root};
use std::ops::Range;

pub(crate) use indent::formatted_slice;

use crate::source::Edit as E;
type Edit<D> = E<<D as Doc>::Source>;

mod indent;
mod structural;
mod template;

pub use crate::source::Content;
pub use template::{TemplateFix, TemplateFixError};

/// Generate replacement content for matched AST nodes.
///
/// The `Replacer` trait defines how to transform a matched node into new content.
/// Implementations can use template strings with meta-variables, structural
/// replacement with other AST nodes, or custom logic.
///
/// # Type Parameters
///
/// - `D: Doc` - The document type containing source code and language information
///
/// # Example Implementation
///
/// ```rust,no_run
/// # use thread_ast_engine::replacer::Replacer;
/// # use thread_ast_engine::{Doc, NodeMatch};
/// # use thread_ast_engine::meta_var::Underlying;
/// struct CustomReplacer;
///
/// impl<D: Doc> Replacer<D> for CustomReplacer {
///     fn generate_replacement(&self, nm: &NodeMatch<'_, D>) -> Underlying<D> {
///         // Custom replacement logic here
///         "new_code".as_bytes().to_vec()
///     }
/// }
/// ```
pub trait Replacer<D: Doc> {
    /// Generate replacement content for a matched node.
    ///
    /// Takes a [`NodeMatch`] containing the matched node and its captured
    /// meta-variables, then returns the raw bytes that should replace the
    /// matched content in the source code.
    ///
    /// # Parameters
    ///
    /// - `nm` - The matched node with captured meta-variables
    ///
    /// # Returns
    ///
    /// Raw bytes representing the replacement content
    fn generate_replacement(&self, nm: &NodeMatch<'_, D>) -> Underlying<D>;

    /// Determine the exact range of source code to replace.
    ///
    /// By default, replaces the entire matched node's range. Some matchers
    /// may want to replace only a portion of the matched content.
    ///
    /// # Parameters
    ///
    /// - `nm` - The matched node
    /// - `matcher` - The matcher that found this node (may provide custom range info)
    ///
    /// # Returns
    ///
    /// Byte range in the source code to replace
    fn get_replaced_range(&self, nm: &NodeMatch<'_, D>, matcher: impl Matcher) -> Range<usize> {
        let range = nm.range();
        if let Some(len) = matcher.get_match_len(nm.get_node().clone()) {
            range.start..range.start + len
        } else {
            range
        }
    }
}

impl<D: Doc> Replacer<D> for str {
    fn generate_replacement(&self, nm: &NodeMatch<'_, D>) -> Underlying<D> {
        template::gen_replacement(self, nm)
    }
}

impl<D: Doc> Replacer<D> for Root<D> {
    fn generate_replacement(&self, nm: &NodeMatch<'_, D>) -> Underlying<D> {
        structural::gen_replacement(self, nm)
    }
}

impl<D, T> Replacer<D> for &T
where
    D: Doc,
    T: Replacer<D> + ?Sized,
{
    fn generate_replacement(&self, nm: &NodeMatch<D>) -> Underlying<D> {
        (**self).generate_replacement(nm)
    }
}

impl<D: Doc> Replacer<D> for Node<'_, D> {
    fn generate_replacement(&self, _nm: &NodeMatch<'_, D>) -> Underlying<D> {
        let range = self.range();
        self.root.doc.get_source().get_range(range).to_vec()
    }
}

#[derive(Debug, Clone)]
enum MetaVarExtract {
    /// $A for captured meta var
    Single(MetaVariableID),
    /// $$$A for captured ellipsis
    Multiple(MetaVariableID),
    Transformed(MetaVariableID),
}

impl MetaVarExtract {
    fn used_var(&self) -> &str {
        match self {
            Self::Single(s) |
            Self::Multiple(s) |
            Self::Transformed(s) => s,
        }
    }
}

fn split_first_meta_var(
    src: &str,
    meta_char: char,
    transform: &[MetaVariableID],
) -> Option<(MetaVarExtract, usize)> {
    debug_assert!(src.starts_with(meta_char));
    let mut i = 0;
    let mut skipped = 0;
    let is_multi = loop {
        i += 1;
        skipped += meta_char.len_utf8();
        if i == 3 {
            break true;
        }
        if !src[skipped..].starts_with(meta_char) {
            break false;
        }
    };
    // no Anonymous meta var allowed, so _ is not allowed
    let i = src[skipped..]
        .find(|c: char| !is_valid_meta_var_char(c))
        .unwrap_or(src.len() - skipped);
    // no name found
    if i == 0 {
        return None;
    }
    let name = src[skipped..skipped + i].to_string();
    let var = if is_multi {
        MetaVarExtract::Multiple(name)
    } else if transform.contains(&name) {
        MetaVarExtract::Transformed(name)
    } else {
        MetaVarExtract::Single(name)
    };
    Some((var, skipped + i))
}
