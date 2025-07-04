# thread

High-performance code analysis and parsing engine powered by tree-sitter for making AI more useful.

## 🚧 Development Status

This crate is currently under active, early development. The API is not yet stable and will change significantly in upcoming versions.

## 🎯 Planned Features

- **Multi-language Support**: Tree-sitter based parsing for Rust, Python, TypeScript, and more. Easily add support for new languages from existing Tree-sitter grammars.
- **Content-Addressable Storage**: Efficient deduplication and versioning of code elements
- **High Performance**: Concurrent processing optimized for speed, memory usage, and uncompromised performance
- **WASM Ready**: Compile to WebAssembly for edge deployments and browser integration
- **Incremental Parsing**: Live updates for language server capabilities
- **Query System**: Flexible, language-agnostic code element extraction

## 🥅 Design Goals

- **Tree-sitter-Powered**: 🌳Tree-sitter provides a foundation that already meets many of our goals - it's fast, efficient, supports incremental parsing, a wide range of languages, and is very tolerant of syntax errors. We're really glad to be able to build on it.
- **Extremely Fast**: While tree-sitter is very fast, our needs for Thread are bigger than most uses of tree-sitter today. We want Thread to scale to very large codebases without anyone needing to rack up a large AWS bill. Part of that means being very smart about what we parse, how we parse it, and what we actually keep in memory. We want Thread to be able to parse and analyze large codebases in seconds, not minutes or hours. Our specific targets:
  - Single file parsing: <10ms for typical files
  - Project-wide parsing: <100ms for medium projects, <1s for large projects, <1 lifetime for [mega projects](https://research.google/pubs/why-google-stores-billions-of-lines-of-code-in-a-single-repository/)
  - Incremental updates: <1ms for small changes
  - WASM overhead: <5ms additional latency
- **Rich Data, Small Package**: We want Thread to provide a lot of information about code in a very simple and easy-to-use format that both AI assistants and humans can understand.
- **Modular**: While we are building Thread for a very specific purpose, we know that its core functionality will be useful for many other projects. That's why we've committed to keeping it open source and very modular.
- **High level**: Thread will have some necessary low-level components and complex technical details, but we want to keep the high-level API as simple and easy to use as possible. We want Thread to be a tool that anyone can use, not just experts in code analysis or parsing.

## 🏗️ Architecture

Thread will be a modular workspace:

- `thread` - Main entrypoint crate
  - `thread-core` - Core traits and types
  - `thread-parser` - High-performance parsing engine
  - `thread-query` - Query system for code extraction
  - `thread-languages` - Language-specific implementations
  - `thread-wasm` - WebAssembly bindings
  - `thread-cli` - Command-line interface

## 🛠️ Current Crate Status

- [] `thread-core`: Core traits and types for Thread
  - [x] Basic data structures
  - [x] Core traits
  - [x] Basic error handling
- [] `thread-cli`: Command-line interface for Thread
  - Not started yet
- [] `thread-derive` - Derive macros for Thread
  - [X] Basic outline of the derive macros
- [] `thread-http`: HTTP client for Thread to interact with outside LLMs
  - Not started yet
- [] `thread-languages`: Language-specific implementations
  - [] Rust
  - [] Python
  - [] TypeScript (planned initial language support)
  - Not started yet. We will likely save this for after we get the derive macros working, which will allow us to easily add support for new languages.
- [] `thread-parser`: High-performance parsing engine
  - [x] Basic sketch/scaffold of the parser
- [] `thread-query`: Query system for code extraction
  - [x] Basic scaffold of the query system
- [] `thread-wasm`: WebAssembly bindings
  - Not started yet

## 📄 Licensing

Thread is dual-licensed to serve both open source and commercial users:

### Open Source License

- **AGPL-3.0-or-later** for open source projects
- The AGPL strictly keeps the code open source and free for everyone to use it how they want.
- If you don't change the code or build something on top of it, you can use it freely in your open source or even commercial projects.
- If you modify the code or build something on top of it, you must also open source your changes and the entire project under the AGPL.
- Perfect for research, learning, and open source development
- If you don't want to open source your project, we've got you covered with a commercial license option:

### Commercial License

- **Proprietary license** available for commercial use
- Allows integration into proprietary software and as part of networked services without open source requirements
- Contact [licensing@knitli.com](mailto:licensing@knitli.com) for licensing terms and pricing
- Ideal for businesses, startups, and commercial applications who don't want to open source their code

### The *Focus* Service built on Thread

- Thread is part of our planned **Focus** service, which will deliver the power of Thread -- and much more -- as a fast, edge-first, hosted service.
- Perfect for teams who need the best from AI coding assistants without managing infrastructure, and want to focus on building their products.
- We'll also offer a free tier for public projects, so you can use Thread without needing to run your own servers.

## About Thread

### 🏢 About Thread 🧵

[Knitli](https://knitli.com) actively develops Thread. We break down barriers between powerful AI technology and everyday people. We believe the most powerful AI systems *must* be accessible, understandable, and usable for everyone. As part of that vision, we need AI systems that meet people *where they are*, and don't require people to adapt to the technology. For the development community, that means coding assistants who actually can readily understand the codebase, and all current internal and external APIs. Thread, as part of our planned **Focus** service, aims to provide that understanding, which is just the start of our vision for AI accessibility.

We got the idea for Thread while working on a completely different product for our launch, and got frustrated with the ham-handedness of today's AI coding assistants. It turns out that it's not just people who need tools that meet them where they are. That's what Thread is all about: freeing AI assistants from their own limitations, so they can be more useful to developers.

### Contributing

We welcome contributions of all kinds! Whether you're a seasoned developer or just getting started, your help is appreciated. Please read our [contributing guidelines](https://github.com/knitli/thread/blob/main/CONTRIBUTING.md).

### API

Thread is in active early development. We expect the API to change a lot in upcoming versions. We will provide documentation as we stabilize the API, but for now, please refer to the source code and examples in the repository and expect that it will all break with updates. The main `thread` crate will be the entrypoint for using Thread in your projects, with all supporting crates feature-gated from there. As we stabilize the API, we expect to only guarantee the API of the `thread` crate, and may break the API of the supporting crates in future versions (we will use major version numbers to make that clear when we do it).

### Minimum Supported Rust Version (MSRV)

We currently have MSRV set to 1.85, which is very nearly the current Rust stable (1.88). We didn't want to start from a place of supporting legacy Rust, but haven't decided on an actual MSRV policy. If you need a specific MSRV or have a suggestion, please open an issue. We can probably support older versions of Rust, but didn't want to worry about it as we got started.

## 📞 Contact

- **General**: [team@knitli.com](mailto:team@knitli.com)
- **Licensing**: [licensing@knitli.com](mailto:licensing@knitli.com)
- **Development**: Follow progress on GitHub

---

*Thread is part of the knitli ecosystem of AI accessibility tools.*
