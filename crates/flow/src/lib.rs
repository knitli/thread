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

pub mod bridge;
pub mod flows;
pub mod functions;
pub mod runtime;
pub mod sources;
pub mod targets;

// Re-exports
pub use bridge::CocoIndexAnalyzer;
pub use flows::builder::ThreadFlowBuilder;
pub use runtime::{EdgeStrategy, LocalStrategy, RuntimeStrategy};
