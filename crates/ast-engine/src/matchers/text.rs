// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

//! # Text-Based Pattern Matching
//!
//! Provides regex-based matchers for finding AST nodes by their text content.
//! Useful when you need to match nodes based on their actual text rather
//! than their structural properties.
//!
//! ## Core Types
//!
//! - [`RegexMatcher`] - Matches nodes whose text content matches a regex pattern
//! - [`RegexMatcherError`] - Errors from invalid regex patterns
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! // Find all nodes containing specific text patterns
//! let number_matcher = RegexMatcher::try_new(r"\d+")?; // Numbers
//! let email_matcher = RegexMatcher::try_new(r"[\w\.-]+@[\w\.-]+\.\w+")?; // Emails
//!
//! // Find all numeric literals
//! let numbers: Vec<_> = root.find_all(&number_matcher).collect();
//!
//! // Find specific variable names
//! let temp_vars = RegexMatcher::try_new(r"temp\w*")?;
//! let temp_variables: Vec<_> = root.find_all(&temp_vars).collect();
//! ```
//!
//! ## Use Cases
//!
//! Text matching complements structural patterns when you need to:
//! - Find nodes with specific naming patterns
//! - Locate hardcoded values or literals
//! - Search for code smells in text content
//! - Filter nodes by complex text criteria

use super::matcher::Matcher;
use crate::Doc;
use crate::Node;
use crate::meta_var::MetaVarEnv;

use bit_set::BitSet;
use regex::{Error as RegexError, Regex};
use thiserror::Error;

use std::borrow::Cow;

/// Errors that can occur when creating a [`RegexMatcher`].
#[derive(Debug, Error)]
pub enum RegexMatcherError {
    /// The provided regex pattern is invalid.
    ///
    /// Common causes include unbalanced parentheses, invalid escape sequences,
    /// or unsupported regex features.
    #[error("Parsing text matcher fails.")]
    Regex(#[from] RegexError),
}

/// Matcher that finds AST nodes based on regex patterns applied to their text content.
///
/// `RegexMatcher` enables flexible text-based searching within AST nodes.
/// It matches any node whose text content satisfies the provided regular expression.
///
/// # Examples
///
/// ```rust,ignore
/// // Match numeric literals
/// let numbers = RegexMatcher::try_new(r"^\d+$")?;
/// let numeric_nodes: Vec<_> = root.find_all(&numbers).collect();
///
/// // Find TODO comments
/// let todos = RegexMatcher::try_new(r"(?i)todo|fixme")?;
/// let todo_comments: Vec<_> = root.find_all(&todos).collect();
///
/// // Match specific naming patterns
/// let private_vars = RegexMatcher::try_new(r"^_\w+")?;
/// let private_variables: Vec<_> = root.find_all(&private_vars).collect();
/// ```
///
/// # Performance Note
///
/// Text matching requires extracting text from every tested node, which can be
/// slower than structural matching. Consider combining with other matchers
/// or using more specific patterns when possible.
#[derive(Clone, Debug)]
pub struct RegexMatcher {
    /// Compiled regex pattern for matching node text
    regex: Regex,
}

impl RegexMatcher {
    pub fn try_new(text: &str) -> Result<Self, RegexMatcherError> {
        Ok(Self {
            regex: Regex::new(text)?,
        })
    }
}

impl Matcher for RegexMatcher {
    fn match_node_with_env<'tree, D: Doc>(
        &self,
        node: Node<'tree, D>,
        _env: &mut Cow<MetaVarEnv<'tree, D>>,
    ) -> Option<Node<'tree, D>> {
        self.regex.is_match(&node.text()).then_some(node)
    }

    fn potential_kinds(&self) -> Option<BitSet> {
        None
    }
}
