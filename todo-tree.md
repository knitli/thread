<!--
SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Working Notes

```plaintext
└─ thread
   ├─ crates
   │  ├─ core
   │  │  └─ ast_grep.rs
   │  │     └─ line 19: TODO : evaluate usage of tree-sitter dependent features after initial release
   │  ├─ engine
   │  │  ├─ analyzer.rs
   │  │  │  └─ line 37: TODO : This is the Day 2 deliverable target
   │  │  └─ lib.rs
   │  │     ├─ line 41: TODO : Implement file analysis
   │  │     └─ line 59: TODO : Track files
   │  ├─ languages
   │  │  ├─ src
   │  │  │  ├─ bash.rs
   │  │  │  │  ├─ line 30: TODO
   │  │  │  │  └─ line 43: TODO : change the replacer to log $A
   │  │  │  ├─ lib.rs
   │  │  │  │  └─ line 729: TODO : add test for file_types
   │  │  │  └─ php.rs
   │  │  │     └─ line 23: TODO : better php support
   │  │  └─ Cargo.toml
   │  │     ├─ line 29: TODO : Add toml, markdown, pkl
   │  │     └─ line 31: TODO : Integrate tree-sitter wasmtime support
   │  ├─ parse
   │  │  └─ rust_parser.rs
   │  │     ├─ line 154: TODO : detect actual visibility
   │  │     ├─ line 155: TODO : detect async
   │  │     ├─ line 156: TODO : detect generics
   │  │     ├─ line 157: TODO : extract doc comments
   │  │     ├─ line 208: TODO : Parse the ITEMS list properly
   │  │     └─ line 256: TODO : implement export detection
   │  └─ wasm
   │     └─ Cargo.toml
   │        └─ line 18: TODO : Add compile-for-os support as a universal binary (i.e. for moonrepo)
```

## Early Todos

- Standardize feature flags across the crates to focus on use case vice *feature*
  - Features by build type (i.e. Production/Release)
  - By deployment (i.e. local/CI as CLI vs cloud/wasm worker vs browser vs wasi CLI app)
  - By IO (filesystem vs network)
  - Storage type (i.e. in-memory vs on-disk vs database)
  - Separate serde deps throughout

- Implement Tower Services to provide common execution and storage abstractions. Features will just enable/disable implementing backends.
