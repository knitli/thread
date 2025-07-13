// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

mod utils;

#[cfg_attr(feature = "serialization", derive(serde::Serialize))]
struct WasmAnalysisResult {
    node_count: usize,
    edge_count: usize,
    language: String,
    line_count: usize,
}
