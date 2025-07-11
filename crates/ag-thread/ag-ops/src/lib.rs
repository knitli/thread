/*!
This module contains advanced pattern operations for ast-grep.

It provides APIs for composing and combining patterns, including:
- Boolean operations (And, Or, Not)
- Pattern aggregation (All, Any)
- Pattern composition utilities
- Advanced matcher combinators
*/

mod ops;

// Re-export implemented traits and types
pub use ops::{And, Or, Not, All, Any, Op};
