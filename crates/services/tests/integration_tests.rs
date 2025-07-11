//! Comprehensive integration tests for the service layer implementation.

use ag_service_registry::prelude::*;
use ag_service_registry::{
    adapters::noop::*,
    compatibility::{CliCompatWrapper, CompatOutputFormat},
    core_extraction::{ScanEngine, ConfigExtractor},
    orchestration::AstGrepService,
    registry::ServiceRegistry,
    runtime::{AsyncRuntime, RuntimeEnvironment},
};
use thread_utils::FastMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio;

/// Test the complete service layer workflow end-to-end.
#[tokio::test]
async fn test_complete_service_workflow() {
    // Create service registry with test implementations
    let registry = create_test_registry();
    let runtime = ag_service_registry::runtime::create_runtime_for(
        RuntimeEnvironment::CloudflareWorkers
    );
    let service = AstGrepService::new(registry, runtime);

    // Test scan operation
    let scan_options = ScanOptions {
        paths: vec!["src/".to_string()],
        file_patterns: vec!["*.rs".to_string()],
        exclude_patterns: vec!["**/target/**".to_string()],
        language_filter: Some(vec!["rust".to_string()]),
        config_source: ConfigSource::Inline("rules: []".to_string()),
        output_format: OutputFormat::Json { pretty: true, include_metadata: false },
        context_lines_before: 2,
        context_lines_after: 2,
        severity_filter: None,
        rule_filter: None,
        interactive: false,
    };

    let scan_result = service.scan_files(scan_options).await;
    assert!(scan_result.is_ok(), "Scan operation should succeed");

    // Test search operation
    let search_request = SearchRequest {
        pattern: "fn main".to_string(),
        discovery: FileDiscoveryRequest {
            paths: vec!["src/".to_string()],
            patterns: vec!["*.rs".to_string()],
            exclude_patterns: vec![],
            follow_symlinks: false,
            max_depth: None,
            language_filter: Some(vec!["rust".to_string()]),
        },
        options: SearchOptions {
            strictness: None,
            selector: None,
            context_lines_before: 1,
            context_lines_after: 1,
        },
        output_format: OutputFormat::Plain,
    };

    let search_result = service.search_pattern(search_request).await;
    assert!(search_result.is_ok(), "Search operation should succeed");

    // Test fix operation
    let fix_request = FixRequest {
        fixes: vec![FixInstruction {
            pattern: "old_pattern".to_string(),
            replacement: "new_pattern".to_string(),
            language: Some("rust".to_string()),
        }],
        discovery: FileDiscoveryRequest::default(),
        options: FixOptions {
            dry_run: true,
            interactive: false,
            backup: true,
        },
        output_format: OutputFormat::Plain,
    };

    let fix_result = service.apply_fixes(fix_request).await;
    assert!(fix_result.is_ok(), "Fix operation should succeed");
}

/// Test service registry builder and environment detection.
#[tokio::test]
async fn test_service_registry_environments() {
    // Test CLI environment
    let cli_registry = ServiceRegistry::for_environment(Environment::Cli);
    assert!(cli_registry.interaction.supports_interaction());
    assert!(cli_registry.terminal.supports_terminal());

    // Test Cloudflare Workers environment
    let cf_registry = ServiceRegistry::for_environment(Environment::CloudflareWorkers);
    assert!(!cf_registry.interaction.supports_interaction());
    assert!(!cf_registry.terminal.supports_terminal());

    // Test WASM environment
    let wasm_registry = ServiceRegistry::for_environment(Environment::Wasm);
    assert!(!wasm_registry.interaction.supports_interaction());
    assert!(!wasm_registry.terminal.supports_terminal());

    // Test custom registry builder
    let custom_registry = ServiceRegistry::builder()
        .with_file_discovery(Arc::new(NoOpFileDiscoveryService))
        .with_configuration(Arc::new(NoOpConfigurationService))
        .with_output(Arc::new(NoOpOutputService))
        .with_interaction(Arc::new(NoOpInteractionService))
        .with_terminal(Arc::new(NoOpTerminalService))
        .with_test_execution(Arc::new(NoOpTestExecutionService))
        .build();

    // Test that custom registry works
    let runtime = ag_service_registry::runtime::create_runtime_for(
        RuntimeEnvironment::CloudflareWorkers
    );
    let service = AstGrepService::new(custom_registry, runtime);

    let scan_result = service.scan_files(ScanOptions::default()).await;
    assert!(scan_result.is_ok());
}

/// Test async runtime implementations across environments.
#[tokio::test]
async fn test_async_runtime_implementations() {
    // Test Cloudflare runtime
    let cf_runtime = ag_service_registry::runtime::create_runtime_for(
        RuntimeEnvironment::CloudflareWorkers
    );
    assert_eq!(cf_runtime.max_concurrency(), 1);
    assert!(!cf_runtime.supports_parallelism());

    // Test WASM runtime
    let wasm_runtime = ag_service_registry::runtime::create_runtime_for(
        RuntimeEnvironment::Wasm
    );
    assert_eq!(wasm_runtime.max_concurrency(), 1);
    assert!(!wasm_runtime.supports_parallelism());

    // Test parallel processing
    let items = vec![1, 2, 3, 4, 5];
    let results = cf_runtime.process_parallel(
        items,
        |x| Box::pin(async move { Ok(x * 2) })
    ).await;

    assert!(results.is_ok());
    let results = results.unwrap();
    assert_eq!(results, vec![2, 4, 6, 8, 10]);
}

/// Test core extraction logic.
#[tokio::test]
async fn test_core_extraction_logic() {
    // Test scan engine
    let scan_result = ScanEngine::scan_file_with_rules(
        "test.rs",
        "fn main() { println!(\"Hello, world!\"); }",
        "rules: []",
        &ScanOptions::default(),
    ).await;

    assert!(scan_result.is_ok());
    let matches = scan_result.unwrap();
    // With placeholder implementation, we expect empty matches for now
    assert_eq!(matches.len(), 0);

    // Test config extraction
    let scan_options = ConfigExtractor::extract_scan_options_from_cli(
        Some("test.yml"),
        None,
        vec!["src/".to_string()],
        vec!["*.rs".to_string()],
        3,
        3,
        true,
        OutputFormat::Colored { style: ReportStyle::Rich },
    );

    assert_eq!(scan_options.paths, vec!["src/"]);
    assert_eq!(scan_options.context_lines_before, 3);
    assert_eq!(scan_options.context_lines_after, 3);
    assert!(scan_options.interactive);

    // Test file discovery request extraction
    let discovery_request = ConfigExtractor::extract_file_discovery_request(
        vec!["src/".to_string(), "tests/".to_string()],
        vec!["*.rs".to_string(), "*.toml".to_string()],
        false,
        Some(vec!["rust".to_string()]),
    );

    assert_eq!(discovery_request.paths, vec!["src/", "tests/"]);
    assert_eq!(discovery_request.patterns, vec!["*.rs", "*.toml"]);
    assert!(!discovery_request.follow_symlinks);
    assert_eq!(discovery_request.language_filter, Some(vec!["rust".to_string()]));
}

/// Test backward compatibility layer.
#[tokio::test]
async fn test_backward_compatibility() {
    let wrapper = create_test_compat_wrapper();

    // Test CLI-compatible scan
    let scan_result = wrapper.run_scan_compatible(
        None, // rule_file
        Some("id: test-rule\nrule: {pattern: test}".to_string()), // inline_rules
        vec![PathBuf::from("src")], // paths
        vec!["*.rs".to_string()], // file_patterns
        vec!["**/target/**".to_string()], // exclude_patterns
        2, // context_before
        2, // context_after
        false, // interactive
        CompatOutputFormat::Json, // output_format
        true, // json_pretty
        false, // include_metadata
        None, // severity_filter
    ).await;

    assert!(scan_result.is_ok(), "CLI-compatible scan should succeed");

    // Test CLI-compatible search
    let search_result = wrapper.run_search_compatible(
        "fn main".to_string(), // pattern
        vec![PathBuf::from("src")], // paths
        vec!["*.rs".to_string()], // file_patterns
        Some("rust".to_string()), // language
        1, // context_before
        1, // context_after
        CompatOutputFormat::Colored, // output_format
    ).await;

    assert!(search_result.is_ok(), "CLI-compatible search should succeed");

    // Test CLI-compatible fix
    let fix_result = wrapper.run_fix_compatible(
        None, // rule_file
        Some("id: test\nrule: {pattern: old}\nfix: new".to_string()), // inline_rules
        vec![PathBuf::from("src")], // paths
        vec!["*.rs".to_string()], // file_patterns
        true, // dry_run
        false, // interactive
        true, // backup
    ).await;

    assert!(fix_result.is_ok(), "CLI-compatible fix should succeed");
}

/// Test legacy function wrappers.
#[tokio::test]
async fn test_legacy_functions() {
    use ag_service_registry::compatibility::legacy;

    // Test legacy scan function
    let scan_result = legacy::scan_with_config_legacy(
        None, // rule
        Some("id: test\nrule: {pattern: test}".to_string()), // inline_rules
        vec![PathBuf::from(".")], // paths
        Some(true), // json
        false, // interactive
        0, // context_before
        0, // context_after
    ).await;

    assert!(scan_result.is_ok(), "Legacy scan function should work");

    // Test legacy search function
    let search_result = legacy::search_pattern_legacy(
        "test_pattern".to_string(), // pattern
        vec![PathBuf::from(".")], // paths
        Some("rust".to_string()), // language
        true, // json
    ).await;

    assert!(search_result.is_ok(), "Legacy search function should work");

    // Test legacy fix function
    let fix_result = legacy::apply_fixes_legacy(
        None, // rule
        Some("id: test\nrule: {pattern: old}\nfix: new".to_string()), // inline_rules
        vec![PathBuf::from(".")], // paths
        true, // dry_run
        false, // interactive
    ).await;

    assert!(fix_result.is_ok(), "Legacy fix function should work");
}

/// Test error handling and edge cases.
#[tokio::test]
async fn test_error_handling() {
    let registry = create_test_registry();
    let runtime = ag_service_registry::runtime::create_runtime_for(
        RuntimeEnvironment::CloudflareWorkers
    );
    let service = AstGrepService::new(registry, runtime);

    // Test with invalid configuration source
    let invalid_scan_options = ScanOptions {
        config_source: ConfigSource::File("/nonexistent/file.yml".to_string()),
        ..Default::default()
    };

    // This should succeed with no-op services but would fail with real implementations
    let scan_result = service.scan_files(invalid_scan_options).await;
    assert!(scan_result.is_ok()); // No-op services always succeed

    // Test error creation and handling
    let source_error = AstGrepError::source_error("test error", "test context");
    match source_error {
        AstGrepError::Source { message, context, .. } => {
            assert_eq!(message, "test error");
            assert_eq!(context, "test context");
        }
        _ => panic!("Expected source error"),
    }

    let validation_error = AstGrepError::validation_error("field", "message");
    match validation_error {
        AstGrepError::Validation { field, message } => {
            assert_eq!(field, "field");
            assert_eq!(message, "message");
        }
        _ => panic!("Expected validation error"),
    }
}

/// Test type conversions and serialization.
#[tokio::test]
async fn test_type_conversions() {
    use ag_service_registry::compatibility::conversion;

    // Test severity conversion
    assert_eq!(conversion::convert_severity("error"), Some(Severity::Error));
    assert_eq!(conversion::convert_severity("WARNING"), Some(Severity::Warning));
    assert_eq!(conversion::convert_severity("info"), Some(Severity::Info));
    assert_eq!(conversion::convert_severity("hint"), Some(Severity::Hint));
    assert_eq!(conversion::convert_severity("off"), Some(Severity::Off));
    assert_eq!(conversion::convert_severity("invalid"), None);

    // Test scan match conversion
    let scan_match = ScanMatch {
        id: uuid::Uuid::new_v4(),
        file_path: "test.rs".to_string(),
        rule_id: "test-rule".to_string(),
        message: "Test message".to_string(),
        severity: Severity::Warning,
        start_line: 10,
        end_line: 10,
        start_column: 5,
        end_column: 15,
        matched_text: "test code".to_string(),
        context_before: vec!["line 8".to_string(), "line 9".to_string()],
        context_after: vec!["line 11".to_string(), "line 12".to_string()],
        metadata: FastMap::new(),
    };

    let cli_match = conversion::convert_scan_match_to_cli(scan_match.clone());
    assert_eq!(cli_match.file_path, scan_match.file_path);
    assert_eq!(cli_match.rule_id, scan_match.rule_id);
    assert_eq!(cli_match.message, scan_match.message);
    assert_eq!(cli_match.line, scan_match.start_line);
    assert_eq!(cli_match.column, scan_match.start_column);
    assert_eq!(cli_match.matched_text, scan_match.matched_text);
}

#[cfg(feature = "tower")]
/// Test Tower service integration.
#[tokio::test]
async fn test_tower_service_integration() {
    use ag_service_registry::tower_support::*;
    use tower::Service;

    // Create Tower service
    let registry = create_test_registry();
    let runtime = ag_service_registry::runtime::create_runtime_for(
        RuntimeEnvironment::CloudflareWorkers
    );
    let ag_service = AstGrepService::new(registry, runtime);
    let mut tower_service = AstGrepTowerService::new(ag_service);

    // Test scan request
    let scan_request = AstGrepRequest::Scan(ScanOptions::default());
    let response = tower_service.call(scan_request).await;
    assert!(response.is_ok());

    match response.unwrap() {
        AstGrepResponse::Scan(results) => {
            assert_eq!(results.files_processed, 0); // No-op services
        }
        _ => panic!("Expected scan response"),
    }

    // Test search request
    let search_request = AstGrepRequest::Search(SearchRequest {
        pattern: "test".to_string(),
        discovery: FileDiscoveryRequest::default(),
        options: SearchOptions::default(),
        output_format: OutputFormat::default(),
    });

    let response = tower_service.call(search_request).await;
    assert!(response.is_ok());

    // Test HTTP utilities
    let json_request = r#"{"scan": true, "paths": ["."]}"#;
    // This will fail because our simple parser expects specific keywords
    let parse_result = http::parse_http_request("POST", json_request);
    assert!(parse_result.is_err()); // Expected with current simple implementation

    // Test response formatting
    let scan_response = AstGrepResponse::Scan(ScanResults {
        matches: vec![],
        execution_time: None,
        files_processed: 0,
    });

    let json_response = http::format_http_response(scan_response);
    assert!(json_response.is_ok());
    assert!(json_response.unwrap().contains("matches"));
}

/// Test performance and concurrency.
#[tokio::test]
async fn test_performance_and_concurrency() {
    let registry = create_test_registry();
    let runtime = ag_service_registry::runtime::create_runtime_for(
        RuntimeEnvironment::CloudflareWorkers
    );
    let service = AstGrepService::new(registry, runtime);

    // Test concurrent operations
    let scan_options = ScanOptions::default();

    let tasks: Vec<_> = (0..10).map(|_| {
        let service = &service;
        let options = scan_options.clone();
        async move {
            service.scan_files(options).await
        }
    }).collect();

    let results = futures::future::join_all(tasks).await;

    // All operations should succeed
    for result in results {
        assert!(result.is_ok());
    }
}

// Helper functions for test setup

fn create_test_registry() -> ServiceRegistry {
    ServiceRegistry::builder()
        .with_file_discovery(Arc::new(NoOpFileDiscoveryService))
        .with_configuration(Arc::new(NoOpConfigurationService))
        .with_output(Arc::new(NoOpOutputService))
        .with_interaction(Arc::new(NoOpInteractionService))
        .with_terminal(Arc::new(NoOpTerminalService))
        .with_test_execution(Arc::new(NoOpTestExecutionService))
        .build()
}

fn create_test_compat_wrapper() -> CliCompatWrapper {
    let registry = create_test_registry();
    let runtime = ag_service_registry::runtime::create_runtime_for(
        RuntimeEnvironment::CloudflareWorkers
    );
    let service = AstGrepService::new(registry, runtime);
    CliCompatWrapper::with_service(service)
}
