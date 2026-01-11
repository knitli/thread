// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

#[cfg(test)]
mod tests {
    use crate::hash_help::{hash_bytes, hash_bytes_with_seed, hash_file, hash_file_with_seed};
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hash_bytes() {
        let data = b"hello world";
        let hash = hash_bytes(data);
        assert_ne!(hash, 0);

        // Deterministic check for rapidhash v3 default seed
        // rapidhash_v3(b"hello world", DEFAULT_RAPID_SECRETS) -> 3397907815814400320
        // (This value comes from rapidhash docs, let's verify it matches)
        assert_eq!(hash, 3397907815814400320);
    }

    #[test]
    fn test_hash_bytes_with_seed() {
        let data = b"hello world";
        let seed = 0x123456;
        let hash = hash_bytes_with_seed(data, seed);

        let hash2 = hash_bytes_with_seed(data, seed);
        assert_eq!(hash, hash2);

        let seed2 = 0x654321;
        let hash3 = hash_bytes_with_seed(data, seed2);
        assert_ne!(hash, hash3);
    }

    #[test]
    fn test_hash_file() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"hello file").unwrap();
        file.flush().unwrap();

        let mut file_reopened = file.reopen().unwrap();
        let hash = hash_file(&mut file_reopened).unwrap();

        // Compare with bytes
        let bytes_hash = hash_bytes(b"hello file");
        assert_eq!(hash, bytes_hash);
    }

    #[test]
    fn test_hash_file_with_seed() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"hello seeded file").unwrap();
        file.flush().unwrap();

        let seed = 987654321;
        let mut file_reopened = file.reopen().unwrap();
        let hash = hash_file_with_seed(&mut file_reopened, seed).unwrap();

        let bytes_hash = hash_bytes_with_seed(b"hello seeded file", seed);
        assert_eq!(hash, bytes_hash);
    }
}
