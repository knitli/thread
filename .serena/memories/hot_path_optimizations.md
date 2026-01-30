<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.

SPDX-License-Identifier: AGPL-3.0-or-later
-->

# Hot Path Optimizations (Phase 3)

## Completed Optimizations

### 1. Pattern Compilation Cache (matcher.rs)
- **Location**: `crates/ast-engine/src/matcher.rs`
- **Mechanism**: Thread-local `HashMap<(String, TypeId), Pattern>` with 256-entry capacity
- **Hot path**: `impl Matcher for str` now calls `cached_pattern_try_new()` instead of `Pattern::try_new()` directly
- **Impact**: Eliminates redundant tree-sitter parsing when same pattern string is used repeatedly (typical in rule scanning)
- **Benchmark**: ~5% improvement in pattern_conversion_optimized; near-zero overhead for cache hits vs precompiled patterns

### 2. String Interning (MetaVariableID -> Arc<str>)
- **Location**: `crates/ast-engine/src/meta_var.rs` (primary), ripple through `replacer.rs`, `match_tree/match_node.rs`, `matchers/pattern.rs`, `rule-engine/rule_core.rs`, `rule-engine/check_var.rs`, `rule-engine/fixer.rs`
- **Change**: `pub type MetaVariableID = String` -> `pub type MetaVariableID = Arc<str>`
- **Impact**: All MetaVarEnv operations (clone, insert, lookup) benefit from Arc<str> semantics
  - Clone: atomic increment (~1ns) vs String clone (~10-50ns)
  - MetaVarEnv clone: 107ns for full env with Arc<str> keys
- **Benchmark**: env_clone_with_arc_str: 107ns per environment clone

### 3. Enhanced Benchmarks
- **Location**: `crates/ast-engine/benches/performance_improvements.rs`
- Added: pattern_cache (cold/warm/precompiled), env_clone_cost, multi_pattern_scanning
- Validates both optimizations with realistic workloads

## Files Modified
- `crates/ast-engine/src/matcher.rs` - pattern cache + imports
- `crates/ast-engine/src/meta_var.rs` - MetaVariableID type + all usages
- `crates/ast-engine/src/replacer.rs` - Arc import + split_first_meta_var
- `crates/ast-engine/src/replacer/template.rs` - with_transform signature + test
- `crates/ast-engine/src/match_tree/match_node.rs` - try_get_ellipsis_mode + match_ellipsis
- `crates/ast-engine/benches/performance_improvements.rs` - new benchmarks
- `crates/rule-engine/src/rule_core.rs` - constraints type
- `crates/rule-engine/src/check_var.rs` - constraints type
- `crates/rule-engine/src/fixer.rs` - Arc conversion for keys

## Test Results
- thread-ast-engine: 142/142 passed, 4 skipped
- thread-rule-engine: 165/168 passed, 3 failed (pre-existing), 2 skipped
