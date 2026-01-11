// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later
//! Hash map, set, and related hashing utilities.
//!
//! Thread uses [`rapidhash::RapidHashMap`] and [`rapidhash::RapidHashSet`] as stand-ins for
//! `std::collections::HashMap` and `std::collections::HashSet` (they ARE `std::collections::HashMap` and
//! `std::collections::HashSet`, but using the [`rapidhash::fast::RandomState`] hash builder.)
//!
//! For Thread's expected workloads, it's *very fast* and sufficiently secure for our needs.
//! // Important to note that `rapidhash` is not a cryptographic hash, and while it's a high quality hash that's optimal in most ways, it hasn't been thoroughly tested for `HashDoD` resistance.
//! For how we use it, this isn't a concern. We also use random seeds for the hash builder, so it should be resistant to hash collision attacks.

use rapidhash::fast::RandomState;

// export RapidHasher for use as a type
pub use rapidhash::fast::RapidHasher as RapidInlineHasher;

// These are effectively aliases for `rapidhash::RapidHashMap` and `rapidhash::RapidHashSet`
// They're less of a mouthful, and we avoid type aliasing a type alias
/// A type alias for `[rapidhash::RapidHashMap]`.
pub type RapidMap<K, V> = rapidhash::RapidHashMap<K, V>;
/// A type alias for `[rapidhash::RapidHashSet]`.
pub type RapidSet<T> = rapidhash::RapidHashSet<T>;

/// Creates a new `RapidMap` with the specified capacity; returning the initialized map for use.
#[inline(always)]
#[must_use] pub fn map_with_capacity<K, V>(capacity: usize) -> RapidMap<K, V>
where
    K: std::hash::Hash + Eq,
    V: Default,
{
    RapidMap::with_capacity_and_hasher(capacity, RandomState::default())
}

/// Creates a new `RapidInlineHashSet` with the specified capacity; returning the initialized set for use.
#[inline(always)]
#[must_use] pub fn set_with_capacity<T>(capacity: usize) -> RapidSet<T>
where
    T: std::hash::Hash + Eq,
{
    RapidSet::with_capacity_and_hasher(capacity, RandomState::default())
}

/// Returns a new `RapidMap` with default values.
#[inline(always)]
#[must_use] pub fn get_map<K, V>() -> RapidMap<K, V> {
    RapidMap::default()
}

/// Returns a new `RapidSet` with default values (a [`rapidhash::RapidHashSet`]).
#[inline(always)]
#[must_use] pub fn get_set<T>() -> RapidSet<T> {
    RapidSet::default()
}

/// Computes a hash for a [`std::fs::File`] object using `rapidhash`.
#[inline(always)]
pub fn hash_file(file: &mut std::fs::File) -> Result<u64, std::io::Error> {
    rapidhash::v3::rapidhash_v3_file(file).map_err(std::io::Error::other)
}

/// Computes a hash for a [`std::fs::File`] object using `rapidhash` with a specified seed.
pub fn hash_file_with_seed(file: &mut std::fs::File, seed: u64) -> Result<u64, std::io::Error> {
    let secrets = rapidhash::v3::RapidSecrets::seed(seed);
    rapidhash::v3::rapidhash_v3_file_seeded(file, &secrets)
        .map_err(std::io::Error::other)
}

/// Computes a hash for a byte slice using `rapidhash`.
#[inline(always)]
#[must_use] pub const fn hash_bytes(bytes: &[u8]) -> u64 {
    rapidhash::v3::rapidhash_v3(bytes)
}

/// Computes a hash for a byte slice using `rapidhash` with a specified seed.
#[inline(always)]
#[must_use] pub const fn hash_bytes_with_seed(bytes: &[u8], seed: u64) -> u64 {
    // Note: RapidSecrets::seed is const, so this should be fine in a const fn
    let secrets = rapidhash::v3::RapidSecrets::seed(seed);
    rapidhash::v3::rapidhash_v3_seeded(bytes, &secrets)
}
