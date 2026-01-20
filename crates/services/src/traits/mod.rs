// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # Service Layer Traits
//!
//! Core traits that define the service layer interfaces for Thread.
//! These traits abstract over ast-grep functionality while preserving
//! all its powerful capabilities and enabling codebase-level intelligence.

pub mod analyzer;
pub mod parser;

#[cfg(feature = "storage-traits")]
pub mod storage;

pub use analyzer::{AnalyzerCapabilities, CodeAnalyzer};
pub use parser::{CodeParser, ParserCapabilities};

#[cfg(feature = "storage-traits")]
pub use storage::{CacheService, StorageService};
