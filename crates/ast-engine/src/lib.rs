// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT
//! # thread-ast-engine
//!
//! **Core AST engine for Thread: parsing, matching, and transforming code using AST patterns.**
//!
//! ## Overview
//!
//! `thread-ast-engine` provides powerful tools for working with Abstract Syntax Trees (ASTs).
//! Forked from [`ast-grep-core`](https://github.com/ast-grep/ast-grep/), it offers language-agnostic
//! APIs for code analysis and transformation.
//!
//! ### What You Can Do
//!
//! - **Parse** source code into ASTs using [tree-sitter](https://tree-sitter.github.io/tree-sitter/)
//! - **Search** for code patterns using flexible meta-variables (like `$VAR`)
//! - **Transform** code by replacing matched patterns with new code
//! - **Navigate** AST nodes with intuitive tree traversal methods
//!
//! Perfect for building code linters, refactoring tools, and automated code modification systems.
//!
//! ## Quick Start
//!
//! Add to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! thread-ast-engine = { version = "0.1.0", features = ["parsing", "matching"] }
//! ```
//!
//! ### Basic Example: Find and Replace Variables
//!
//! ```rust,no_run
//! use thread_ast_engine::Language;
//! use thread_ast_engine::tree_sitter::LanguageExt;
//!
//! // Parse JavaScript/TypeScript code
//! let mut ast = Language::Tsx.ast_grep("var a = 1; var b = 2;");
//!
//! // Replace all 'var' declarations with 'let'
//! ast.replace("var $NAME = $VALUE", "let $NAME = $VALUE")?;
//!
//! // Get the transformed code
//! println!("{}", ast.generate());
//! // Output: "let a = 1; let b = 2;"
//! # Ok::<(), String>(())
//! ```
//!
//! ### Finding Code Patterns
//!
//! ```rust,no_run
//! use thread_ast_engine::matcher::MatcherExt;
//! # use thread_ast_engine::Language;
//! # use thread_ast_engine::tree_sitter::LanguageExt;
//!
//! let ast = Language::Tsx.ast_grep("function add(a, b) { return a + b; }");
//! let root = ast.root();
//!
//! // Find all function declarations
//! if let Some(func) = root.find("function $NAME($$$PARAMS) { $$$BODY }") {
//!     println!("Function name: {}", func.get_env().get_match("NAME").unwrap().text());
//! }
//!
//! // Find all return statements
//! for ret_stmt in root.find_all("return $EXPR") {
//!     println!("Returns: {}", ret_stmt.get_env().get_match("EXPR").unwrap().text());
//! }
//! ```
//!
//! ### Working with Meta-Variables
//!
//! Meta-variables capture parts of the matched code:
//!
//! - `$VAR` - Captures a single AST node
//! - `$$$ITEMS` - Captures multiple consecutive nodes (ellipsis)
//! - `$_` - Matches any node but doesn't capture it
//!
//! ```rust,no_run
//! # use thread_ast_engine::Language;
//! # use thread_ast_engine::tree_sitter::LanguageExt;
//! # use thread_ast_engine::matcher::MatcherExt;
//! let ast = Language::Tsx.ast_grep("console.log('Hello', 'World', 123)");
//! let root = ast.root();
//!
//! if let Some(call) = root.find("console.log($$$ARGS)") {
//!     let args = call.get_env().get_multiple_matches("ARGS");
//!     println!("Found {} arguments", args.len()); // Output: Found 3 arguments
//! }
//! ```
//!
//! ## Core Components
//!
//! ### [`Node`] - AST Navigation
//! Navigate and inspect AST nodes with methods like [`Node::children`], [`Node::parent`], and [`Node::find`].
//!
//! ### [`Pattern`] - Code Matching
//! Match code structures using tree-sitter patterns with meta-variables.
//!
//! ### [`MetaVarEnv`] - Variable Capture
//! Store and retrieve captured meta-variables from pattern matches.
//!
//! ### [`Replacer`] - Code Transformation
//! Replace matched code with new content, supporting template-based replacement.
//!
//! ### [`Language`] - Language Support
//! Abstract interface for different programming languages via tree-sitter grammars.
//!
//! ## Feature Flags
//!
//! - **`parsing`** - Enables tree-sitter parsing (includes tree-sitter dependency)
//! - **`matching`** - Enables pattern matching and node replacement/transformation engine.
//!
//! Use `default-features = false` to opt out of all features and enable only what you need:
//!
//! ```toml
//! [dependencies]
//! thread-ast-engine = { version = "0.1.0", default-features = false, features = ["matching"] }
//! ```
//!
//! ## Advanced Examples
//!
//! ### Custom Pattern Matching
//!
//! ```rust,no_run
//! use thread_ast_engine::ops::Op;
//! # use thread_ast_engine::Language;
//! # use thread_ast_engine::tree_sitter::LanguageExt;
//! # use thread_ast_engine::matcher::MatcherExt;
//!
//! // Combine multiple patterns with logical operators
//! let pattern = Op::either("let $VAR = $VALUE")
//!     .or("const $VAR = $VALUE")
//!     .or("var $VAR = $VALUE");
//!
//! let ast = Language::Tsx.ast_grep("const x = 42;");
//! let root = ast.root();
//!
//! if let Some(match_) = root.find(pattern) {
//!     println!("Found variable declaration");
//! }
//! ```
//!
//! ### Tree Traversal
//!
//! ```rust,no_run
//! # use thread_ast_engine::Language;
//! # use thread_ast_engine::tree_sitter::LanguageExt;
//! # use thread_ast_engine::matcher::MatcherExt;
//! let ast = Language::Tsx.ast_grep("if (condition) { doSomething(); } else { doOther(); }");
//! let root = ast.root();
//!
//! // Traverse all descendants
//! for node in root.dfs() {
//!     if node.kind() == "identifier" {
//!         println!("Identifier: {}", node.text());
//!     }
//! }
//!
//! // Check relationships between nodes
//! if let Some(if_stmt) = root.find("if ($COND) { $$$THEN }") {
//!     println!("If statement condition: {}",
//!         if_stmt.get_env().get_match("COND").unwrap().text());
//! }
//! ```
//!
//! ## License
//!
//! Original ast-grep code is licensed under the [MIT license](./LICENSE-MIT),
//! all changes introduced in this project are licensed under the [AGPL-3.0-or-later](./LICENSE-AGPL-3.0-or-later).
//!
//! See [`VENDORED.md`](crates/ast-engine/VENDORED.md) for more information on our fork, changes, and reasons.

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

#[cfg(feature = "matching")]
mod match_tree;
#[cfg(feature = "matching")]
pub mod matcher;
pub mod meta_var;
#[cfg(feature = "matching")]
pub mod ops;
#[doc(hidden)]
pub mod pinned;
#[cfg(feature = "matching")]
pub mod replacer;

// Re-exports

// the bare types with no implementations
#[cfg(not(feature = "matching"))]
pub use matchers::{
    MatchStrictness, Pattern, PatternBuilder, PatternError, PatternNode,
    matcher::{Matcher, MatcherExt, NodeMatch},
};

// implemented types
#[cfg(feature = "matching")]
pub use matcher::{
    MatchAll, MatchNone, Matcher, MatcherExt, NodeMatch, Pattern, PatternBuilder, PatternError,
    PatternNode,
};

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
