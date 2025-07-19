// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later
//! Module imports for pattern matching. Feature gated except for unimplemented `types` module.
//!
//! ## Implementation Notes
//!
//! We changed the structure here from Ast-Grep, which uses a pattern like what's still
//! in [`crate::replacer`], where the root `parent.rs` module contains all
//! the submodules.
//!
//! ### Why this structure?
//!
//! We needed to access the type definitions without the `matching` feature flag, so we:
//! - Moved type definitions to `types.rs` (which we created).
//! - renamed the directory from `matcher` to `matchers`
//! - Created this `mod.rs` to import the submodules conditionally based on the `matching` feature flag.
//! - Kept trait implementations behind the feature flag.
//! - Moved [`types::MatchStrictness`] to `types.rs` in this module from `crate::match_tree::strictness` (not the implementation, just the type definition).
//!
//! #### Practical Implications
//!
//! From an API perspective, nothing changed -- `matcher` is still the main entry point for pattern matching (if the feature is enabled).

#[cfg(feature = "matching")]
<<<<<<< Updated upstream
use crate::matcher::Matcher;

#[cfg(feature = "matching")]
pub(crate) mod pattern;

#[cfg(feature = "matching")]
pub(crate) mod kind;

#[cfg(feature = "matching")]
pub(crate) mod node_match;

#[cfg(feature = "matching")]
pub(crate) mod text;

pub(crate) mod types;
#[cfg(not(feature = "matching"))]
pub use types::*;
||||||| Stash base
=======
pub(crate) mod pattern;

#[cfg(feature = "matching")]
pub(crate) mod kind;

#[cfg(feature = "matching")]
pub(crate) mod node_match;

#[cfg(feature = "matching")]
pub(crate) mod text;

pub(crate) mod types;
#[cfg(not(feature = "matching"))]
pub use types::*;

pub(crate) mod matcher {
    pub use super::types::Matcher;
}
>>>>>>> Stashed changes
