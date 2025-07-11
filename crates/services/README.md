# ag-service-registry

Service registry and dependency injection system for ag-thread multi-environment deployment.

## Overview

This crate provides a comprehensive service-oriented architecture that enables ast-grep functionality to operate seamlessly across diverse environments including CLI, Cloudflare Workers, CI/CD pipelines, and customer on-premise deployments.

## Key Features

- **Environment-Agnostic**: Works across CLI, cloud, WASM, and CI/CD environments
- **Dependency Injection**: Clean service registry with builder pattern
- **Async-First**: Built with async/await for cloud compatibility
- **Tower Integration**: Middleware support and HTTP service composition
- **Extensible**: Easy to add new service implementations and environments

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   AstGrepService│────│ ServiceRegistry │────│ Environment     │
│   (Orchestrator)│    │                 │    │ Adapters        │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         │              ┌────────┴────────┐             │
         │              │                 │             │
         ▼              ▼                 ▼             ▼
┌─────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────┐
│AsyncRuntime │ │FileDiscovery │ │Configuration │ │   ...    │
│             │ │   Service    │ │   Service    │ │          │
└─────────────┘ └──────────────┘ └──────────────┘ └──────────┘
```

## Core Services

### Service Traits

- **FileDiscoveryService**: File discovery and content reading
- **ConfigurationService**: Rule and configuration management
- **OutputService**: Result formatting and output
- **InteractionService**: User interaction and prompting
- **TerminalService**: Terminal operations and control
- **TestExecutionService**: Test discovery and execution

### Runtime Abstractions

- **AsyncRuntime**: Environment-specific async execution
- **RuntimeEnvironment**: Configuration for different deployment targets

## Usage Examples

### Basic Service Creation

```rust
use ag_service_registry::prelude::*;

// Create service for CLI environment
let service = AstGrepService::for_environment(Environment::Cli);

// Scan files
let options = ScanOptions::default();
let results = service.scan_files(options).await?;
```

### Custom Service Configuration

```rust
use ag_service_registry::prelude::*;

// Build custom service registry
let registry = ServiceRegistry::builder()
    .with_file_discovery(Arc::new(MyCustomFileService::new()))
    .with_configuration(Arc::new(MyCustomConfigService::new()))
    .with_output(Arc::new(MyCustomOutputService::new()))
    .build();

// Create service with custom runtime
let runtime = create_runtime_for(RuntimeEnvironment::Tokio {
    max_concurrency: Some(8)
});

let service = AstGrepService::new(registry, runtime);
```

### Cloudflare Workers Deployment

```rust
use ag_service_registry::prelude::*;

// Create service optimized for CF Workers
let service = AstGrepService::for_environment(Environment::CloudflareWorkers);

// Process request
let request = SearchRequest {
    pattern: "console.log".to_string(),
    discovery: FileDiscoveryRequest::default(),
    options: SearchOptions::default(),
    output_format: OutputFormat::Json { pretty: true, include_metadata: false },
};

let results = service.search_pattern(request).await?;
```

### Tower Service Integration

```rust
use ag_service_registry::tower_support::*;
use tower::ServiceBuilder;

// Create base service
let service = AstGrepService::for_environment(Environment::Cli);

// Wrap with Tower middleware
let tower_service = ServiceBuilder::new()
    .timeout(Duration::from_secs(30))
    .rate_limit(100, Duration::from_secs(60))
    .service(AstGrepTowerService::new(service));
```

## Environment Support

| Environment | File Discovery | Configuration | Output | Interaction |
|-------------|----------------|---------------|--------|-------------|
| **CLI** | Filesystem + ignore | sgconfig.yml | Terminal/JSON | Interactive prompts |
| **Cloudflare Workers** | KV Store | Remote API | HTTP Response | No-op |
| **CI/CD** | Git workspace | API/Files | Annotations | No-op |
| **WASM** | In-memory | Embedded | Return values | No-op |

## Features

### Default Features
- Basic service abstractions
- No-op implementations
- Core runtime support

### Optional Features
- `tokio-runtime`: Full Tokio async runtime support
- `tower`: Tower service and middleware integration
- `cli`: CLI-specific service implementations
- `cloudflare`: Cloudflare Workers service implementations
- `ci`: CI/CD service implementations
- `wasm`: WASM service implementations

## Implementation Status

### ✅ Completed
- Service trait abstractions
- Service registry and builder
- Async runtime abstractions
- No-op service implementations
- Tower service integration
- Basic orchestration layer

### 🚧 In Progress
- CLI service implementations (stubbed)
- Cloudflare Workers implementations (stubbed)
- CI/CD service implementations (stubbed)
- WASM service implementations (stubbed)

### 📋 Planned
- Actual ast-grep integration in orchestration layer
- Middleware implementations (caching, metrics, etc.)
- Performance optimizations
- Error recovery and resilience patterns

## Testing

The crate includes comprehensive tests for all service abstractions:

```bash
cargo test                    # Run all tests
cargo test --features tower  # Test with Tower integration
```

### Mock Services

Use the provided no-op services for testing:

```rust
use ag_service_registry::adapters::noop::*;

let registry = ServiceRegistry::builder()
    .with_file_discovery(Arc::new(NoOpFileDiscoveryService))
    .with_configuration(Arc::new(NoOpConfigurationService))
    .with_output(Arc::new(NoOpOutputService))
    .build();
```

## Migration from CLI

The service layer is designed to be backward compatible with existing CLI code:

```rust
// Before: CLI-coupled
sg scan --rule my-rule.yml src/

// After: Service-based
let options = ScanOptions {
    paths: vec!["src/".to_string()],
    config_source: ConfigSource::File("my-rule.yml".to_string()),
    ..Default::default()
};
service.scan_files(options).await?;
```

## Performance Considerations

- **Async by default**: All operations use async/await for scalability
- **Environment-optimized**: Runtimes adapt to deployment environment constraints
- **Configurable concurrency**: Adjust parallelism based on environment capabilities
- **Minimal overhead**: Service abstraction adds minimal runtime cost

## Contributing

When implementing new service adapters:

1. Implement all required service traits
2. Add comprehensive tests
3. Update environment registry defaults
4. Document environment-specific considerations
5. Add integration tests

## License

MIT - See LICENSE file for details.
