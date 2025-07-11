# Copilot Instructions for Thread

## Project Overview

Thread is a Rust code analysis engine that provides intelligent context for AI assistants. The project is transitioning from vendored ast-grep CLI code to a multi-environment service architecture supporting CLI, cloud, WASM, and CI/CD deployments.

## Architecture Guidelines

### Service Layer Pattern
The codebase follows a service abstraction pattern to support multiple environments:

```rust
// Pure service functions (environment-agnostic)
pub async fn scan_with_services(
    file_discovery: Arc<dyn FileDiscoveryService>,
    config_service: Arc<dyn ConfigurationService>,
    options: ScanOptions,
) -> Result<ScanResults>
```

### Crate Organization

- **ag-thread/**: Vendored ast-grep modules being refactored for service layers
- **thread-core/**: Core traits, types, and errors (pure abstractions)
- **thread-engine/**: Main analysis implementation using petgraph
- **thread-parse/**: AST-grep integration and language detection
- **Service-ready crates**: `ag-core`, `ag-search`, `ag-fix`, `ag-types`, `ag-label`
- **Needs refactoring**: `ag-scan`, `ag-utils`, `ag-check-rule` (heavy CLI dependencies)

## Development Commands

Essential commands for this workspace:

```bash
# Build all crates (except WASM)
mise run build
mise run b

# WASM builds
mise run build-wasm          # Development (single-threaded)
mise run build-wasm-release  # Production optimized

# Testing and quality
mise run test               # Tests with cargo nextest
mise run lint               # Full linting via hk run check
mise run ci                 # All CI checks
```

## Key Patterns to Follow

### 1. Service Trait Definitions
When creating new services, follow the pattern from `ag-types`:

```rust
#[async_trait]
pub trait YourService: Send + Sync {
    async fn your_method(&self, input: &str) -> Result<Output>;
}
```

### 2. Environment-Agnostic Core Functions
Avoid CLI dependencies in core logic:

```rust
// ✅ Good: Pure function with injected services
pub async fn analyze_with_services(
    content: String,
    services: &ServiceRegistry
) -> Result<AnalysisResult>

// ❌ Avoid: Direct filesystem or terminal access
pub fn analyze_files(paths: Vec<PathBuf>) -> Result<()>
```

### 3. Multi-Environment Support
Structure implementations for different environments:

```rust
// CLI implementation
impl YourService for CliYourService { /* uses std::fs */ }

// Cloud implementation  
impl YourService for CloudYourService { /* uses S3/HTTP */ }

// WASM implementation
impl YourService for WasmYourService { /* uses fetch API */ }
```

## CLI Dependencies Analysis Status

Refer to individual `CLI_DEPENDENCIES.md` files in each ag-thread crate:

- **Immediate attention needed**: `ag-scan/`, `ag-utils/` (heavy CLI dependencies)
- **Service-ready**: `ag-core/`, `ag-search/`, `ag-fix/`, `ag-types/`, `ag-label/`
- **Minor refactoring**: `ag-rule/`, `ag-check-rule/`

## Critical Abstractions

### File Operations
Replace direct filesystem access with service traits:
```rust
// Instead of std::fs::read_to_string
let content = file_service.read_file(path).await?;
```

### Terminal I/O  
Replace direct terminal access with service traits:
```rust
// Instead of println! or crossterm
output_service.write(&format!("Result: {}", result)).await?;
```

### Configuration Loading
Replace direct file config loading:
```rust
// Instead of reading YAML files directly
let rules = config_service.load_rules(ConfigSource::Path(path)).await?;
```

## Testing Strategy

- Use `cargo nextest -j 1` for parallel tests with race condition prevention
- Mock service implementations for unit tests
- Environment-specific integration tests for each service implementation
- `RUST_BACKTRACE=1` enabled for debugging

## WASM Considerations

- Default builds are single-threaded for Cloudflare Workers compatibility
- Core logic separated from filesystem operations for WASM portability
- Multi-threaded builds available for browser environments (`--multi-threading`)

When working with WASM targets, ensure no direct filesystem or process dependencies in core libraries.

## Current Development Focus

**Week 1 Sprint**: Establishing service layer foundations
- Refactoring `ag-scan` to use service abstractions
- Creating `ag-services` crate with core trait definitions
- Implementing CLI service adapters to maintain current functionality

The goal is to enable Thread to analyze code and provide AI-friendly context across all deployment environments while maintaining the performance and functionality of the original ast-grep implementation.
