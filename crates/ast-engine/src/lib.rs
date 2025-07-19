// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

/*!
This module contains the core engine for Thread.

It provides APIs for parsing, traversing, searching and replacing tree-sitter nodes.
The functionality is feature-gated to allow for selective compilation:
- `parsing`: Enables tree-sitter parsing backend
- `matching`: Enables pattern matching and replacement capabilities
*/

pub mod language;
pub mod source;

// Core AST functionality (always available)
mod node;
pub use node::{Node, Position};
pub use source::Doc;
// pub use matcher::types::{MatchStrictness, Pattern, PatternBuilder, PatternError, PatternNode};

// Feature-gated modules
#[cfg(feature = "parsing")]
pub mod tree_sitter;

// Everything but types feature gated behind "matching" in `matchers`
mod matchers;

<<<<<<< Updated upstream
#[cfg(feature = "matching")]
||||||| Stash base
#[cfg(feature = "matching")]
pub mod matcher;
#[cfg(feature = "matching")]
=======
>>>>>>> Stashed changes
pub mod meta_var;
#[cfg(feature = "matching")]
pub mod ops;
#[cfg(feature = "matching")]
pub mod replacer;
#[cfg(feature = "matching")]
pub mod matcher;
#[cfg(feature = "matching")]
mod match_tree;
#[doc(hidden)]
pub mod pinned;

// Re-exports

// the bare types with no implementations
#[cfg(not(feature = "matching"))]
pub use matchers::{matcher::Matcher, Pattern, PatternBuilder, PatternError, PatternNode, MatchStrictness};

// implemented types
#[cfg(feature = "matching")]
pub use matcher::{Matcher, MatcherExt, MatchAll, MatchNone, Pattern, PatternBuilder, PatternError, PatternNode, NodeMatch};

pub use meta_var::MetaVarEnv;

#[cfg(feature = "matching")]
pub use match_tree::MatchStrictness;

pub use language::Language;

pub use node::Root;

pub type AstGrep<D> = Root<D>;

#[cfg(all(test, feature = "parsing", feature = "matching"))]
mod test {
  use super::*;
  use crate::tree_sitter::LanguageExt;
  use language::Tsx;
  use ops::Op;

  pub type Result = std::result::Result<(), String>;

  #[test]
  fn test_replace() -> Result {
    let mut ast_grep = Tsx.ast_grep("var a = 1; let b = 2;");
    ast_grep.replace("var $A = $B", "let $A = $B")?;
    let source = ast_grep.generate();
    assert_eq!(source, "let a = 1; let b = 2;"); // note the semicolon
    Ok(())
  }

  #[test]
  fn test_replace_by_rule() -> Result {
    let rule = Op::either("let a = 123").or("let b = 456");
    let mut ast_grep = Tsx.ast_grep("let a = 123");
    let replaced = ast_grep.replace(rule, "console.log('it works!')")?;
    assert!(replaced);
    let source = ast_grep.generate();
    assert_eq!(source, "console.log('it works!')");
    Ok(())
  }

  #[test]
  fn test_replace_unnamed_node() -> Result {
    // ++ and -- is unnamed node in tree-sitter javascript
    let mut ast_grep = Tsx.ast_grep("c++");
    ast_grep.replace("$A++", "$A--")?;
    let source = ast_grep.generate();
    assert_eq!(source, "c--");
    Ok(())
  }

  #[test]
  fn test_replace_trivia() -> Result {
    let mut ast_grep = Tsx.ast_grep("var a = 1 /*haha*/;");
    ast_grep.replace("var $A = $B", "let $A = $B")?;
    let source = ast_grep.generate();
    assert_eq!(source, "let a = 1 /*haha*/;"); // semicolon

    let mut ast_grep = Tsx.ast_grep("var a = 1; /*haha*/");
    ast_grep.replace("var $A = $B", "let $A = $B")?;
    let source = ast_grep.generate();
    assert_eq!(source, "let a = 1; /*haha*/");
    Ok(())
  }

  #[test]
  fn test_replace_trivia_with_skipped() -> Result {
    let mut ast_grep = Tsx.ast_grep("return foo(1, 2,) /*haha*/;");
    ast_grep.replace("return foo($A, $B)", "return bar($A, $B)")?;
    let source = ast_grep.generate();
    assert_eq!(source, "return bar(1, 2) /*haha*/;"); // semicolon
    Ok(())
  }
}
