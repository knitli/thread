# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/claude-code) when working with code in this repository.

## Build and Test Commands

This project uses [uv](https://docs.astral.sh/uv/) for Python project management.

### Building

```bash
uv run maturin develop   # Build Rust code and install Python package (required after Rust changes)
```

### Testing

```bash
cargo test               # Run Rust tests
uv run dmypy run         # Type check Python code (uses mypy daemon)
uv run pytest python/    # Run Python tests (use after both Rust and Python changes)
```

### Workflow Summary

| Change Type | Commands to Run |
|-------------|-----------------|
| Rust code only | `uv run maturin develop && cargo test` |
| Python code only | `uv run dmypy run && uv run pytest python/` |
| Both Rust and Python | Run all commands from both categories above |

## Code Structure

```
cocoindex/
├── rust/                       # Rust crates (workspace)
│   ├── cocoindex/              # Main crate - core indexing engine
│   │   └── src/
│   │       ├── base/           # Core types: schema, value, spec, json_schema
│   │       ├── builder/        # Flow/pipeline builder logic
│   │       ├── execution/      # Runtime execution: evaluator, indexer, live_updater
│   │       ├── llm/            # LLM integration
│   │       ├── ops/            # Operations: sources, targets, functions
│   │       ├── service/        # Service layer
│   │       └── setup/          # Setup and configuration
│   └── utils/                  # General utilities: error handling, batching, etc.
│

## Key Concepts

- **CocoIndex** is an data processing framework that maintains derived data from source data incrementally
- **Flows** define data transformation pipelines from sources to targets
- **Operations** (ops) include sources, functions, and targets
- The system supports incremental updates - only reprocessing changed data
