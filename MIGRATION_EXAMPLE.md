# FastMap API Migration Example

This document shows how existing code using `FastMap` can be gradually migrated to use the new unified API.

## Current Code Analysis

Looking at `crates/ast-engine/src/meta_var.rs`, the code currently uses `FastMap` in several ways:

### 1. Basic Usage (No Changes Needed)
```rust
// This continues to work unchanged
let mut map = FastMap::default();
map.insert(key, value);
let exists = map.contains_key(&key);
```

### 2. Direct `.get()` Usage
```rust
// Current code (lines 77, 86, 98, etc.)
pub fn get_match(&self, var: &str) -> Option<&'_ Node<'t, D>> {
    self.single_matched.get(var)  // Returns different types depending on backend
}

pub fn get_multiple_matches_ref(&self, var: &str) -> Option<&Vec<Node<'t, D>>> {
    self.multi_matched.get(var)   // Returns different reference types
}
```

**Migration Options:**

**Option A: Use Unified API (Recommended for new code)**
```rust
use thread_utils::{FastMap, FastMapExt};

// For cases where you need owned values
pub fn get_match_owned(&self, var: &str) -> Option<Node<'t, D>>
where
    Node<'t, D>: Clone
{
    self.single_matched.get_owned(var)  // Consistent across all backends
}
```

**Option B: Keep Existing API (For performance-critical paths)**
```rust
// Keep the existing code unchanged - it still works
pub fn get_match(&self, var: &str) -> Option<&'_ Node<'t, D>> {
    self.single_matched.get(var)  // Native API, backend-specific behavior
}
```

### 3. Iterator Usage
```rust
// Current code (lines 104, 108, 112)
pub fn get_matched_variables(&self) -> impl Iterator<Item = MetaVariable> + use<'_, 't, D> {
    let single = self.single_matched.keys().map(|n| MetaVariable::Capture(n.clone(), false));
    let transformed = self.transformed_var.keys().map(|n| MetaVariable::Capture(n.clone(), false));
    let multi = self.multi_matched.keys().map(|n| MetaVariable::MultiCapture(n.clone()));
    single.chain(multi).chain(transformed)
}
```

**Migration to Unified API:**
```rust
use thread_utils::{FastMap, FastMapExt};

pub fn get_matched_variables(&self) -> impl Iterator<Item = MetaVariable> + use<'_, 't, D> {
    let single = self.single_matched.keys_vec().into_iter()
        .map(|n| MetaVariable::Capture(n, false));
    let transformed = self.transformed_var.keys_vec().into_iter()
        .map(|n| MetaVariable::Capture(n, false));
    let multi = self.multi_matched.keys_vec().into_iter()
        .map(|n| MetaVariable::MultiCapture(n));
    single.chain(multi).chain(transformed)
}
```

### 4. Entry API Usage
```rust
// Current code (line 92)
self.multi_matched
    .entry(label.into())
    .or_default()
    .push(node);
```

**Migration to Unified API:**
```rust
use thread_utils::{FastMap, FastMapEntryExt};

// Option A: Use unified entry API
self.multi_matched
    .fast_entry(label.into())
    .and_modify(|vec| vec.push(node.clone()))
    .or_insert(vec![node]);

// Option B: Keep native entry API (still works)
self.multi_matched
    .entry(label.into())
    .or_default()
    .push(node);
```

### 5. Complex Iteration (From Into trait, lines 314-342)
```rust
// Current code
for (id, node) in env.single_matched {
    ret.insert(id, node.text().into());
}
for (id, bytes) in env.transformed_var {
    ret.insert(id, <D::Source as Content>::encode_bytes(&bytes).to_string());
}
for (id, nodes) in env.multi_matched {
    // ... complex processing
}
```

**Migration to Unified API:**
```rust
use thread_utils::{FastMap, FastMapExt};

// Safer iteration that doesn't hold locks (for DashMap backend)
for (id, node) in env.single_matched.iter_vec() {
    ret.insert(id, node.text().into());
}
for (id, bytes) in env.transformed_var.iter_vec() {
    ret.insert(id, <D::Source as Content>::encode_bytes(&bytes).to_string());
}
for (id, nodes) in env.multi_matched.iter_vec() {
    // ... complex processing
}
```

## Migration Strategy

### Phase 1: Keep Existing Code (✅ Already Done)
- No changes needed
- Existing code continues to work
- Type aliases provide the same interface

### Phase 2: Gradual Migration
- Import extension traits: `use thread_utils::{FastMapExt, FastMapEntryExt};`
- Replace problematic patterns (like iteration) with unified API
- Use unified API for new code

### Phase 3: Full Migration (Optional)
- Use unified API throughout for consistency
- Benefits: Better thread safety, consistent behavior
- Trade-offs: Owned values instead of references

## Specific Migration for MetaVarEnv

Here's how the `MetaVarEnv` could be gradually migrated:

```rust
use thread_utils::{FastMap, FastMapExt, FastMapEntryExt};

impl<'t, D: Doc> MetaVarEnv<'t, D> {
    // Keep existing methods unchanged for backward compatibility
    pub fn get_match(&self, var: &str) -> Option<&'_ Node<'t, D>> {
        self.single_matched.get(var)
    }

    // Add new unified API methods
    pub fn get_match_owned(&self, var: &str) -> Option<Node<'t, D>>
    where
        Node<'t, D>: Clone
    {
        self.single_matched.get_owned(var)
    }

    // Thread-safe iteration for concurrent scenarios
    pub fn get_matched_variables_safe(&self) -> impl Iterator<Item = MetaVariable> {
        let single = self.single_matched.keys_vec().into_iter()
            .map(|n| MetaVariable::Capture(n, false));
        let transformed = self.transformed_var.keys_vec().into_iter()
            .map(|n| MetaVariable::Capture(n, false));
        let multi = self.multi_matched.keys_vec().into_iter()
            .map(|n| MetaVariable::MultiCapture(n));
        single.chain(multi).chain(transformed)
    }

    // Unified entry API for labels
    pub fn add_label_safe(&mut self, label: &str, node: Node<'t, D>)
    where
        Node<'t, D>: Clone
    {
        self.multi_matched
            .fast_entry(label.into())
            .and_modify(|vec| vec.push(node.clone()))
            .or_insert(vec![node]);
    }
}
```

## Benefits of Migration

### Immediate Benefits (Phase 1)
- Zero breaking changes
- Code continues to work across all backends

### Gradual Migration Benefits (Phase 2)
- Better thread safety with DashMap
- Consistent behavior across backends
- No lock-related deadlocks in iteration

### Full Migration Benefits (Phase 3)
- Completely unified codebase
- Easier reasoning about concurrency
- Simplified debugging across different feature flags

## Performance Considerations

### When to Use Native API
- Performance-critical tight loops
- Cases where references are sufficient
- When you need zero-copy access

### When to Use Unified API
- Business logic and general usage
- When thread safety is important
- When consistency across backends matters
- New code development

## Conclusion

The migration can be done gradually without breaking existing functionality. The unified API provides safety and consistency benefits, while the native API remains available for performance-critical scenarios.
