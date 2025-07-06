thread/
├── crates/
│   ├── thread-core/        # Main engine + petgraph + error types
│   ├── thread-parse/       # ast-grep integration
│   ├── thread-store/       # Content store + memory mapping
│   ├── thread-diff/        # Vendored difftastic diff algorithms
│   ├── thread-cli/         # Command line interface
│   └── thread-wasm/        # WASM bindings
├── examples/               # Usage examples
├── docs/                   # Documentation
└── scripts/               # Build scripts


# Delete unnecessary crates
rm -rf crates/ast crates/derive crates/query crates/vcs

# Rename existing crates
mv crates/core crates/thread-core
mv crates/parser crates/thread-parse  
mv crates/buffer crates/thread-store
mv crates/diff crates/thread-diff
mv crates/cli crates/thread-cli
mv crates/wasm crates/thread-wasm

# Merge language detection into parser
cp crates/lang/src/* crates/thread-parse/src/
rm -rf crates/lang
