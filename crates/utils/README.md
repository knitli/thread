<!--
SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# thread-utilities

Shared utilities for the Thread code analysis platform, including SIMD-accelerated string operations
and fast non-cryptographic hashing.

## Overview

`thread-utilities` provides performance-critical building blocks used across the Thread workspace:

- **Fast Hashing** — RapidHash-based maps and sets for high-throughput lookups
- **SIMD String Operations** — Hardware-accelerated character and column detection via `memchr`

## Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `hashers` | RapidHash-based `HashMap`/`HashSet` wrappers and file-hashing helpers | ✅ |
| `simd` | SIMD-accelerated ASCII and column detection utilities | ✅ |

## Usage

```toml
[dependencies]
thread-utilities = { version = "0.1", features = ["hashers", "simd"] }
```

### Fast Hash Collections

```rust
use thread_utilities::{RapidMap, RapidSet, get_map, get_set};

// Faster than std HashMap for string keys
let mut map: RapidMap<String, u64> = get_map();
map.insert("main.rs".into(), 0xdeadbeef);

// Hash a file's contents for content-addressing
let mut file = std::fs::File::open("src/main.rs")?;
let digest = thread_utilities::hash_file(&mut file)?;
```

### SIMD Utilities

```rust
use thread_utilities::{is_ascii_simd, get_char_column_simd};

// Fast ASCII check over a string slice
let all_ascii = is_ascii_simd("hello world");

// Compute column position accounting for multi-byte chars
let col = get_char_column_simd("fn µmain()", 3);
```

## License

AGPL-3.0-or-later

