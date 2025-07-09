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

#[cfg(all(feature = "dashmap", not(feature = "wasm-single-thread")))]
pub type FastMap<K, V> = dashmap::DashMap<K, V>;
#[cfg(all(feature = "dashmap", not(feature = "wasm-single-thread")))]
pub type FastSet<K> = dashmap::DashSet<K>;

// Fallback to HashMap for single-threaded
#[cfg(all(feature = "dashmap", feature = "wasm-single-thread"))]
pub type FastMap<K, V> = rapidhash::RapidHashMap<K, V>;
#[cfg(all(feature = "dashmap", feature = "wasm-single-thread"))]
pub type FastSet<K> = rapidhash::RapidHashSet<K>;

// Fallback to HashMap when dashmap feature is not enabled (should never be the case, but just in case...)
#[cfg(not(feature = "dashmap"))]
pub type FastMap<K, V> = rapidhash::RapidHashMap<K, V>;
#[cfg(not(feature = "dashmap"))]
pub type FastSet<K> = rapidhash::RapidHashSet<K>;
