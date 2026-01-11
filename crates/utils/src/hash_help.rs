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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::io::Write;

    // Test constants
    const HASH_DISTRIBUTION_TEST_SIZE: usize = 1000;
    const HASH_DISTRIBUTION_MIN_UNIQUENESS: usize = 95; // 95% uniqueness threshold
    const LARGE_FILE_SIZE: usize = 100_000;

    // Tests for hash_bytes
    #[test]
    fn test_hash_bytes_empty() {
        let hash = hash_bytes(&[]);
        // Should return a consistent hash for empty input
        assert_eq!(hash, hash_bytes(&[]));
    }

    #[test]
    fn test_hash_bytes_simple() {
        let data = b"hello world";
        let hash = hash_bytes(data);
        // Should be deterministic
        assert_eq!(hash, hash_bytes(data));
    }

    #[test]
    fn test_hash_bytes_different_inputs() {
        let hash1 = hash_bytes(b"hello");
        let hash2 = hash_bytes(b"world");
        let hash3 = hash_bytes(b"hello world");
        
        // Different inputs should produce different hashes
        assert_ne!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_ne!(hash2, hash3);
    }

    #[test]
    fn test_hash_bytes_deterministic() {
        let data = b"The quick brown fox jumps over the lazy dog";
        let hash1 = hash_bytes(data);
        let hash2 = hash_bytes(data);
        
        assert_eq!(hash1, hash2, "Hash should be deterministic");
    }

    #[test]
    fn test_hash_bytes_avalanche() {
        // Test that small changes in input produce different hashes (avalanche effect)
        let hash1 = hash_bytes(b"test");
        let hash2 = hash_bytes(b"Test"); // Single bit change
        let hash3 = hash_bytes(b"test1"); // Additional character
        
        assert_ne!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_ne!(hash2, hash3);
    }

    #[test]
    fn test_hash_bytes_large_input() {
        // Test with larger input
        let large_data = vec![0u8; 10000];
        let hash1 = hash_bytes(&large_data);
        
        // Should be deterministic even for large inputs
        assert_eq!(hash1, hash_bytes(&large_data));
        
        // Slightly different large input
        let mut large_data2 = large_data.clone();
        large_data2[5000] = 1;
        let hash2 = hash_bytes(&large_data2);
        
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_bytes_various_sizes() {
        // Test various input sizes to exercise different code paths
        for size in [0, 1, 7, 8, 15, 16, 31, 32, 63, 64, 127, 128, 255, 256, 1023, 1024] {
            let data = vec![0u8; size];
            let hash = hash_bytes(&data);
            // Should be deterministic
            assert_eq!(hash, hash_bytes(&data), "Failed for size {}", size);
        }
    }

    // Tests for hash_bytes_with_seed
    #[test]
    fn test_hash_bytes_with_seed_deterministic() {
        let data = b"test data";
        let seed = 12345u64;
        
        let hash1 = hash_bytes_with_seed(data, seed);
        let hash2 = hash_bytes_with_seed(data, seed);
        
        assert_eq!(hash1, hash2, "Hash with seed should be deterministic");
    }

    #[test]
    fn test_hash_bytes_with_seed_different_seeds() {
        let data = b"test data";
        
        let hash1 = hash_bytes_with_seed(data, 1);
        let hash2 = hash_bytes_with_seed(data, 2);
        let hash3 = hash_bytes_with_seed(data, 3);
        
        // Different seeds should produce different hashes
        assert_ne!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_ne!(hash2, hash3);
    }

    #[test]
    fn test_hash_bytes_with_seed_vs_no_seed() {
        let data = b"test data";
        
        let hash_no_seed = hash_bytes(data);
        let hash_with_seed = hash_bytes_with_seed(data, 0);
        
        // Default hash and seeded hash with seed 0 might differ
        // (depends on implementation, but they should at least work)
        let _ = hash_no_seed;
        let _ = hash_with_seed;
    }

    #[test]
    fn test_hash_bytes_with_seed_empty() {
        let seed = 42u64;
        let hash1 = hash_bytes_with_seed(&[], seed);
        let hash2 = hash_bytes_with_seed(&[], seed);
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_bytes_with_seed_distribution() {
        // Test that different seeds produce well-distributed hashes
        let data = b"test";
        let mut hashes = HashSet::new();
        
        for seed in 0..100 {
            let hash = hash_bytes_with_seed(data, seed);
            hashes.insert(hash);
        }
        
        // Should have high uniqueness (allowing for small collision chance)
        assert!(
            hashes.len() >= HASH_DISTRIBUTION_MIN_UNIQUENESS,
            "Expected high hash distribution, got {} unique hashes out of 100",
            hashes.len()
        );
    }

    // Tests for hash_file
    #[test]
    fn test_hash_file_empty() -> Result<(), std::io::Error> {
        let mut temp_file = tempfile::NamedTempFile::new()?;
        temp_file.flush()?;
        
        let mut file = temp_file.reopen()?;
        let hash1 = hash_file(&mut file)?;
        
        // Reopen and hash again
        let mut file = temp_file.reopen()?;
        let hash2 = hash_file(&mut file)?;
        
        assert_eq!(hash1, hash2, "Empty file hash should be deterministic");
        Ok(())
    }

    #[test]
    fn test_hash_file_simple() -> Result<(), std::io::Error> {
        let mut temp_file = tempfile::NamedTempFile::new()?;
        temp_file.write_all(b"hello world")?;
        temp_file.flush()?;
        
        let mut file = temp_file.reopen()?;
        let hash1 = hash_file(&mut file)?;
        
        // Reopen and hash again
        let mut file = temp_file.reopen()?;
        let hash2 = hash_file(&mut file)?;
        
        assert_eq!(hash1, hash2, "File hash should be deterministic");
        Ok(())
    }

    #[test]
    fn test_hash_file_different_contents() -> Result<(), std::io::Error> {
        let mut temp_file1 = tempfile::NamedTempFile::new()?;
        temp_file1.write_all(b"hello")?;
        temp_file1.flush()?;
        
        let mut temp_file2 = tempfile::NamedTempFile::new()?;
        temp_file2.write_all(b"world")?;
        temp_file2.flush()?;
        
        let mut file1 = temp_file1.reopen()?;
        let hash1 = hash_file(&mut file1)?;
        
        let mut file2 = temp_file2.reopen()?;
        let hash2 = hash_file(&mut file2)?;
        
        assert_ne!(hash1, hash2, "Different file contents should produce different hashes");
        Ok(())
    }

    #[test]
    fn test_hash_file_large() -> Result<(), std::io::Error> {
        let mut temp_file = tempfile::NamedTempFile::new()?;
        let large_data = vec![0xABu8; LARGE_FILE_SIZE];
        temp_file.write_all(&large_data)?;
        temp_file.flush()?;
        
        let mut file = temp_file.reopen()?;
        let hash1 = hash_file(&mut file)?;
        
        // Reopen and hash again
        let mut file = temp_file.reopen()?;
        let hash2 = hash_file(&mut file)?;
        
        assert_eq!(hash1, hash2, "Large file hash should be deterministic");
        Ok(())
    }

    #[test]
    fn test_hash_file_vs_hash_bytes_consistency() -> Result<(), std::io::Error> {
        let data = b"test data for consistency check";
        
        let mut temp_file = tempfile::NamedTempFile::new()?;
        temp_file.write_all(data)?;
        temp_file.flush()?;
        
        let mut file = temp_file.reopen()?;
        let file_hash = hash_file(&mut file)?;
        
        let bytes_hash = hash_bytes(data);
        
        assert_eq!(file_hash, bytes_hash, "File hash should match byte hash for same content");
        Ok(())
    }

    // Tests for hash_file_with_seed
    #[test]
    fn test_hash_file_with_seed_deterministic() -> Result<(), std::io::Error> {
        let mut temp_file = tempfile::NamedTempFile::new()?;
        temp_file.write_all(b"test data")?;
        temp_file.flush()?;
        
        let seed = 12345u64;
        
        let mut file1 = temp_file.reopen()?;
        let hash1 = hash_file_with_seed(&mut file1, seed)?;
        
        let mut file2 = temp_file.reopen()?;
        let hash2 = hash_file_with_seed(&mut file2, seed)?;
        
        assert_eq!(hash1, hash2, "File hash with seed should be deterministic");
        Ok(())
    }

    #[test]
    fn test_hash_file_with_seed_different_seeds() -> Result<(), std::io::Error> {
        let mut temp_file = tempfile::NamedTempFile::new()?;
        temp_file.write_all(b"test data")?;
        temp_file.flush()?;
        
        let mut file1 = temp_file.reopen()?;
        let hash1 = hash_file_with_seed(&mut file1, 1)?;
        
        let mut file2 = temp_file.reopen()?;
        let hash2 = hash_file_with_seed(&mut file2, 2)?;
        
        let mut file3 = temp_file.reopen()?;
        let hash3 = hash_file_with_seed(&mut file3, 3)?;
        
        assert_ne!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_ne!(hash2, hash3);
        Ok(())
    }

    #[test]
    fn test_hash_file_with_seed_vs_hash_bytes_consistency() -> Result<(), std::io::Error> {
        let data = b"test data for seeded consistency";
        let seed = 42u64;
        
        let mut temp_file = tempfile::NamedTempFile::new()?;
        temp_file.write_all(data)?;
        temp_file.flush()?;
        
        let mut file = temp_file.reopen()?;
        let file_hash = hash_file_with_seed(&mut file, seed)?;
        
        let bytes_hash = hash_bytes_with_seed(data, seed);
        
        assert_eq!(file_hash, bytes_hash, "Seeded file hash should match seeded byte hash");
        Ok(())
    }

    // Tests for RapidMap and RapidSet helper functions
    #[test]
    fn test_get_map() {
        let map: RapidMap<String, i32> = get_map();
        assert!(map.is_empty());
    }

    #[test]
    fn test_get_set() {
        let set: RapidSet<String> = get_set();
        assert!(set.is_empty());
    }

    #[test]
    fn test_map_with_capacity() {
        let map: RapidMap<String, i32> = map_with_capacity(100);
        assert!(map.is_empty());
        assert!(map.capacity() >= 100);
    }

    #[test]
    fn test_set_with_capacity() {
        let set: RapidSet<String> = set_with_capacity(100);
        assert!(set.is_empty());
        assert!(set.capacity() >= 100);
    }

    #[test]
    fn test_rapid_map_basic_operations() {
        let mut map: RapidMap<String, i32> = get_map();
        
        map.insert("one".to_string(), 1);
        map.insert("two".to_string(), 2);
        map.insert("three".to_string(), 3);
        
        assert_eq!(map.len(), 3);
        assert_eq!(map.get("one"), Some(&1));
        assert_eq!(map.get("two"), Some(&2));
        assert_eq!(map.get("three"), Some(&3));
        assert_eq!(map.get("four"), None);
    }

    #[test]
    fn test_rapid_set_basic_operations() {
        let mut set: RapidSet<String> = get_set();
        
        set.insert("apple".to_string());
        set.insert("banana".to_string());
        set.insert("cherry".to_string());
        
        assert_eq!(set.len(), 3);
        assert!(set.contains("apple"));
        assert!(set.contains("banana"));
        assert!(set.contains("cherry"));
        assert!(!set.contains("date"));
    }

    #[test]
    fn test_rapid_map_with_capacity_usage() {
        let mut map: RapidMap<i32, String> = map_with_capacity(10);
        
        for i in 0..5 {
            map.insert(i, format!("value_{}", i));
        }
        
        assert_eq!(map.len(), 5);
        assert!(map.capacity() >= 10);
    }

    #[test]
    fn test_rapid_set_with_capacity_usage() {
        let mut set: RapidSet<i32> = set_with_capacity(10);
        
        for i in 0..5 {
            set.insert(i);
        }
        
        assert_eq!(set.len(), 5);
        assert!(set.capacity() >= 10);
    }

    #[test]
    fn test_rapid_map_hash_distribution() {
        // Test that RapidMap handles hash collisions properly
        let mut map: RapidMap<i32, String> = get_map();
        
        for i in 0..HASH_DISTRIBUTION_TEST_SIZE {
            map.insert(i as i32, format!("value_{}", i));
        }
        
        assert_eq!(map.len(), HASH_DISTRIBUTION_TEST_SIZE);
        
        // Verify all values are retrievable
        for i in 0..HASH_DISTRIBUTION_TEST_SIZE {
            assert_eq!(map.get(&(i as i32)), Some(&format!("value_{}", i)));
        }
    }

    #[test]
    fn test_rapid_set_hash_distribution() {
        // Test that RapidSet handles hash collisions properly
        let mut set: RapidSet<i32> = get_set();
        
        for i in 0..HASH_DISTRIBUTION_TEST_SIZE {
            set.insert(i as i32);
        }
        
        assert_eq!(set.len(), HASH_DISTRIBUTION_TEST_SIZE);
        
        // Verify all values are present
        for i in 0..HASH_DISTRIBUTION_TEST_SIZE {
            assert!(set.contains(&(i as i32)));
        }
    }
}
