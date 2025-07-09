// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later


#[cfg(all(feature = "dashmap", not(feature = "wasm-single-thread")))]
pub type FastMap<K, V> = dashmap::DashMap<K, V>;
#[cfg(all(feature = "dashmap", not(feature = "wasm-single-thread")))]
pub type FastSet<K> = dashmap::DashSet<K>;

// Fallback to HashMap for single-threaded
#[cfg(all(feature = "dashmap", feature = "wasm-single-thread"))]
pub type FastMap<K, V> = std::collections::HashMap<K, V>;
#[cfg(all(feature = "dashmap", feature = "wasm-single-thread"))]
pub type FastSet<K> = std::collections::HashSet<K>;

// Fallback to HashMap when dashmap feature is not enabled (should never be the case, but just in case...)
#[cfg(not(feature = "dashmap"))]
pub type FastMap<K, V> = std::collections::HashMap<K, V>;
#[cfg(not(feature = "dashmap"))]
pub type FastSet<K> = std::collections::HashSet<K>;
