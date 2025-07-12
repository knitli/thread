//! Service registry and dependency injection system for ag-thread multi-environment deployment.
//!
//! This crate provides the service registry, async runtime abstractions, and environment-specific
//! service implementations that enable ast-grep functionality to operate across CLI, cloud,
//! CI/CD, and customer on-premise environments.

#[cfg(feature = "serde")]
pub mod ag_core_integration;
pub mod core_services;
pub mod orchestrator;

#[cfg(feature = "tower")]
pub mod tower_support;

// Re-export key types from ag-service-types
pub use ag_service_types::*;

// Re-export new clean architecture components
#[cfg(feature = "serde")]
pub use ag_core_integration::{detect_language, parse_rule_patterns, AgCoreService, EnrichedMatch};
pub use core_services::{
    CliEnvironmentAdapter, EnvironmentAdapter, FileRuleProvider, OutputFormatter, RuleProvider,
    ServiceRegistry, TerminalFormatter,
};
pub use orchestrator::{AstGrepEngine, ScanRequest};

#[cfg(feature = "tower")]
pub use tower_support::*;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::ag_core_integration::{detect_language, AgCoreService, EnrichedMatch};
    pub use crate::core_services::{
        CliEnvironmentAdapter, EnvironmentAdapter, FileRuleProvider, OutputFormatter, RuleProvider,
        ServiceRegistry, TerminalFormatter,
    };
    pub use crate::orchestrator::{AstGrepEngine, ScanRequest};


    // Common types
    pub use ag_service_types::{
        AstGrepError, ConfigSource, Environment, FixOptions, FixResults, OutputFormat, Result,
        ScanOptions, ScanResults, SearchOptions, SearchResults, Severity,
    };
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Create a service registry for the specified environment
pub fn create_registry_for(environment: Environment) -> ServiceRegistry {
    ServiceRegistry::for_environment(environment)
}

/// Create a service registry with custom configuration
pub fn create_custom_registry() -> ServiceRegistryBuilder {
    ServiceRegistry::builder()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let _registry = create_registry_for(Environment::Cli);
        // Should not panic
    }

    #[test]
    fn test_custom_registry_builder() {
        let _builder = create_custom_registry();
        // Should not panic
    }
}
