// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later
//! FastMap and FastSet types for efficient key-value storage.
//! These types are designed to provide high-performance access to key-value pairs (or values for FastSet) regardless of deployment environment.
//! By default, FastMap and FastSet use [`DashMap`][dashmap::DashMap] and [`DashSet`][dashmap::DashSet], respectively, for their implementations.
//! In single-threaded WASM environments, or if the "dashmap" feature isn't enabled, they fall back to using [`RapidHashMap`][rapidhash::RapidHashMap] and [`RapidHashSet`][rapidhash::RapidHashSet], which are implementations of the std collection's HashMap using the blistering-fast rapidhash algorithm.
//! The API for both *mostly* mirrors HashMap, but you do need to be careful with references to data they contain.
//! DashMap can lock up if you mutate the data (like with `insert`, or `remove`) while there's an internal view on one or more of its keys/values. Basically, if you need to mutate the FastMap, make sure to drop all references first (references are created from methods like `get` or `iter`).
//! Note: You **can** use `FastMap::insert` and `FastMap::remove` while holding a reference to a value if you're using those methods on a *view of FastMap* (like `&my_fast_map: &FastMap<K, V>`) or by cloning it first (of course).

// 'inline' feature here works both ways -- DashMap/DashSets inlining is handled at the feature level, while RapidHashMap/RapidInlineHashMap inlining is handled at the type level.
cfg_if::cfg_if! {
    if #[cfg(all(feature = "dashmap", not(feature = "wasm-single-thread")))] {
        pub type FastMap<K, T> = dashmap::DashMap<K, T>;
        pub type FastSet<T> = dashmap::DashSet<T>;
    } else if #[cfg(all(feature = "dashmap", feature = "wasm-single-thread", not(feature = "inline")))] {
        // Fallback to RapidHashMap for single-threaded, no inline
        pub type FastMap<K, T> = rapidhash::RapidHashMap<K, T>;
        pub type FastSet<T> = rapidhash::RapidHashSet<T>;
    } else if #[cfg(all(feature = "dashmap", feature = "wasm-single-thread", feature = "inline"))] {
        // Fallback to RapidInlineHashMap for single-threaded, inline
        pub type FastMap<K, T> = rapidhash::RapidInlineHashMap<K, T>;
        pub type FastSet<T> = rapidhash::RapidInlineHashSet<T>;
    } else if #[cfg(all(not(feature = "dashmap"), not(feature = "wasm-single-thread"), not(feature = "inline")))] {
        // Fallback to RapidHashMap when dashmap feature is not enabled and not in single-threaded WASM mode, not inline
        pub type FastMap<K, T> = rapidhash::RapidHashMap<K, T>;
        pub type FastSet<T> = rapidhash::RapidHashSet<T>;
    } else {
        // Fallback to RapidInlineHashMap when dashmap feature is not enabled and not in single-threaded WASM mode, inline
        pub type FastMap<K, T> = rapidhash::RapidInlineHashMap<K, T>;
        pub type FastSet<T> = rapidhash::RapidInlineHashSet<T>;
    }
}
