# Thread Code Style & Conventions

## Editor Configuration (.editorconfig)

### Global Defaults
- **Charset**: UTF-8
- **Indent Style**: Spaces
- **Indent Size**: 4 spaces
- **Line Endings**: LF (Unix-style)
- **Trim Trailing Whitespace**: Yes
- **Insert Final Newline**: Yes

### File-Specific Overrides
- **JSON files** (*.json, *.json5, *.jsonc): 2-space indent
- **Makefiles**: Tab indentation (if present)
- **Shell scripts**: Use shfmt conventions (switch_case_indent, space_redirects, etc.)

## Rust Style

### Rustfmt
- Uses default rustfmt configuration
- No custom rustfmt.toml file
- Applied via `cargo fmt --all`

### Clippy Lints (Cargo.toml)
- **Base Level**: `pedantic` and `nursery` warnings enabled
- **Cargo Warnings**: Enabled for workspace-level issues
- **Denied**: `dbg_macro` (no debug macros in production)
- **Key Allowed Lints** (to reduce noise):
  - `cast_*` lints (lossless, truncation, wrapping)
  - `module_name_repetitions`
  - `missing_errors_doc`, `missing_panics_doc`
  - `too_many_lines`
  - `cognitive_complexity` (warning, not error)
  - `todo` (allowed during development)

### Code Organization
- One module per file
- Clear separation of concerns
- Use workspace dependencies defined in root Cargo.toml
- Follow tree-sitter's lint conventions

## Naming Conventions

### Rust Standard Conventions
- **Types/Traits**: PascalCase (e.g., `AstEngine`, `PatternMatcher`)
- **Functions/Variables**: snake_case (e.g., `find_pattern`, `match_node`)
- **Constants**: SCREAMING_SNAKE_CASE (e.g., `MAX_DEPTH`, `DEFAULT_TIMEOUT`)
- **Lifetimes**: Single lowercase letter (e.g., `'a`, `'b`)
- **Type Parameters**: Single uppercase letter or PascalCase (e.g., `T`, `Lang`)

### File Naming
- Module files: snake_case (e.g., `pattern_matcher.rs`)
- Test files: `tests/` directory or `#[cfg(test)]` modules
- Binary crates: snake_case (e.g., `thread_cli`)

## License Headers

### Required on ALL Files
- REUSE-compliant license headers required
- Format:
```rust
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: [Name] <email>
//
// SPDX-License-Identifier: AGPL-3.0-or-later
```
- Or for forked code:
```rust
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT
```
- Update with: `./scripts/update-licenses.py`

## Build Profiles

### Development Profiles
- **dev**: opt-level=1, debug=true, incremental, 256 codegen-units
- **dev-debug**: Uses cranelift backend for faster debug builds
- **release-dev**: Release with debug info, incremental

### Release Profiles
- **release**: opt-level=3, LTO=true, panic=abort, 1 codegen-unit
- **wasm-release**: opt-level=s (size), strip=true, LTO=true

### Dependency Optimization
- Proc-macros always compiled with opt-level=3 even in dev builds

## Testing Conventions

### Test Execution
- Use `cargo nextest` (faster, better output)
- Run with `--all-features` flag
- Single-threaded for consistency: `-j 1`
- No fail-fast during development: `--no-fail-fast`

### Test Organization
- Unit tests: In same file with `#[cfg(test)]` module
- Integration tests: In crate's `tests/` directory
- Test data: In `test_data/` directories
- Benchmarks: In `benches/` directories

## Documentation

### Code Documentation
- Public APIs must have rustdoc comments
- Use `///` for item documentation
- Use `//!` for module documentation
- Examples in rustdoc when helpful

### Project Documentation
- README.md at project root
- CLAUDE.md for AI assistant guidance
- Per-crate documentation in crate README files
