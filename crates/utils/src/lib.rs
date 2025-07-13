// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>

// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(unused_imports)]
#[cfg(feature = "hashers")]
mod hash_help;
#[cfg(feature = "hashers")]
pub use hash_help::{RapidMap, RapidSet, get_map, get_set, map_with_capacity, set_with_capacity,
    hash_file, hash_file_with_seed, hash_bytes, hash_bytes_with_seed};
