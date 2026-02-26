// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # Thread CocoIndex Integration
//!
//! This crate implements the bridge between Thread's imperative library and
//! CocoIndex's declarative dataflow engine.
//!
//! It follows the Service-Library architecture using the following patterns:
//! - **Adapter**: Wraps Thread logic in CocoIndex operators
//! - **Bridge**: Implements thread-services traits using CocoIndex
//! - **Builder**: Constructs analysis flows
//! - **Strategy**: Handles runtime differences (CLI vs Edge)

pub mod batch;
pub mod bridge;
pub mod cache;
pub mod conversion;
pub mod flows;
pub mod functions;
pub mod incremental;
pub mod monitoring;
pub mod registry;
pub mod runtime;
pub mod sources;
pub mod targets;

#[cfg(all(not(feature = "worker"), feature = "mimalloc"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(test)]
// Re-exports
pub use bridge::CocoIndexAnalyzer;
pub use flows::builder::ThreadFlowBuilder;
pub use registry::ThreadOperators;
pub use runtime::{EdgeStrategy, LocalStrategy, RuntimeStrategy};
