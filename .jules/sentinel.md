## 2026-04-08 - [Remove unsafe reference casting from rule registration]
**Vulnerability:** Unsafe pointer casting in `Registration` circumvented `Arc` aliasing rules to produce mutable reference
**Learning:** Complex nested data structures in Rust sometimes invite unsafe hacks to avoid locking overhead, but this exposes undefined behavior and concurrency bugs.
**Prevention:** Use standard concurrency primitives like `Arc<RwLock<T>>` to enforce Rust memory safety guarantees.
