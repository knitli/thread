// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Operator registry for Thread's Recoco integration.
//!
//! This module provides registration functions for all Thread-specific operators
//! using Recoco's ExecutorFactoryRegistry. Operators follow the SimpleFunctionFactoryBase
//! pattern for proper integration with the Recoco dataflow engine.

use recoco::ops::factory_bases::{SimpleFunctionFactoryBase, TargetFactoryBase};
use recoco::ops::sdk::ExecutorFactoryRegistry;
use recoco::prelude::Error as RecocoError;

use crate::functions::{
    calls::ExtractCallsFactory, imports::ExtractImportsFactory, parse::ThreadParseFactory,
    symbols::ExtractSymbolsFactory,
};
use crate::targets::d1::D1TargetFactory;

/// Thread operators available for Recoco flows.
///
/// These operators integrate Thread's semantic code analysis capabilities
/// into Recoco's dataflow engine for incremental, cached code parsing.
///
/// # Available Operators
///
/// ## Functions (Transforms)
///
/// ### `thread_parse`
/// Parse source code into AST with semantic extraction.
///
/// **Inputs**:
/// - `content` (String): Source code content
/// - `language` (String): Language identifier (extension or name)
/// - `file_path` (String, optional): File path for context
///
/// **Output**: Struct with fields:
/// - `symbols` (LTable): Symbol definitions
/// - `imports` (LTable): Import statements
/// - `calls` (LTable): Function calls
///
/// ### `extract_symbols`
/// Extract symbol table from parsed document.
///
/// **Inputs**:
/// - `parsed_document` (Struct): Output from `thread_parse`
///
/// **Output**: LTable with fields:
/// - `name` (String): Symbol name
/// - `kind` (String): Symbol kind (function, class, etc.)
/// - `scope` (String): Scope identifier
///
/// ### `extract_imports`
/// Extract import statements from parsed document.
///
/// **Inputs**:
/// - `parsed_document` (Struct): Output from `thread_parse`
///
/// **Output**: LTable with fields:
/// - `symbol_name` (String): Imported symbol name
/// - `source_path` (String): Import source path
/// - `kind` (String): Import kind
///
/// ### `extract_calls`
/// Extract function calls from parsed document.
///
/// **Inputs**:
/// - `parsed_document` (Struct): Output from `thread_parse`
///
/// **Output**: LTable with fields:
/// - `function_name` (String): Called function name
/// - `arguments_count` (Int64): Number of arguments
///
/// ## Targets (Export Destinations)
///
/// ### `d1`
/// Export data to Cloudflare D1 edge database.
///
/// **Configuration**:
/// - `account_id` (String): Cloudflare account ID
/// - `database_id` (String): D1 database ID
/// - `api_token` (String): Cloudflare API token
/// - `table_name` (String): Target table name
///
/// **Features**:
/// - Content-addressed deduplication via primary key
/// - UPSERT pattern (INSERT ... ON CONFLICT DO UPDATE)
/// - Batch operations for efficiency
/// - Edge-distributed caching
pub struct ThreadOperators;

impl ThreadOperators {
    /// List of all available Thread operator names (functions).
    pub const OPERATORS: &'static [&'static str] = &[
        "thread_parse",
        "extract_symbols",
        "extract_imports",
        "extract_calls",
    ];

    /// List of all available Thread target names (export destinations).
    pub const TARGETS: &'static [&'static str] = &["d1"];

    /// Check if an operator name is a Thread operator.
    pub fn is_thread_operator(name: &str) -> bool {
        Self::OPERATORS.contains(&name)
    }

    /// Check if a target name is a Thread target.
    pub fn is_thread_target(name: &str) -> bool {
        Self::TARGETS.contains(&name)
    }

    /// Register all Thread operators with the provided registry.
    ///
    /// This function creates instances of all Thread operator factories and
    /// registers them using the SimpleFunctionFactoryBase::register() and
    /// TargetFactoryBase::register() methods.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use recoco::ops::sdk::ExecutorFactoryRegistry;
    /// use thread_flow::ThreadOperators;
    ///
    /// let mut registry = ExecutorFactoryRegistry::new();
    /// ThreadOperators::register_all(&mut registry)?;
    /// ```
    pub fn register_all(registry: &mut ExecutorFactoryRegistry) -> Result<(), RecocoError> {
        // Register function operators
        ThreadParseFactory.register(registry)?;
        ExtractSymbolsFactory.register(registry)?;
        ExtractImportsFactory.register(registry)?;
        ExtractCallsFactory.register(registry)?;

        // Register target operators
        D1TargetFactory.register(registry)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator_names() {
        assert!(ThreadOperators::is_thread_operator("thread_parse"));
        assert!(ThreadOperators::is_thread_operator("extract_symbols"));
        assert!(ThreadOperators::is_thread_operator("extract_imports"));
        assert!(ThreadOperators::is_thread_operator("extract_calls"));
        assert!(!ThreadOperators::is_thread_operator("unknown_op"));
    }

    #[test]
    fn test_operator_count() {
        assert_eq!(ThreadOperators::OPERATORS.len(), 4);
    }

    #[test]
    fn test_target_names() {
        assert!(ThreadOperators::is_thread_target("d1"));
        assert!(!ThreadOperators::is_thread_target("unknown_target"));
    }

    #[test]
    fn test_target_count() {
        assert_eq!(ThreadOperators::TARGETS.len(), 1);
    }

    #[test]
    fn test_register_all() {
        let mut registry = ExecutorFactoryRegistry::new();
        // Registration succeeding without error validates that all operators are properly registered
        ThreadOperators::register_all(&mut registry).expect("registration should succeed");
    }
}
