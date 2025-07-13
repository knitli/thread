# Overview

Thread is a library designed to provide intelligent context for AI assistants. It's built on top of `ast-grep`, which uses tree-sitter to parse code. `ast-grep` is primarily designed as a CLI tool, and focused on code identification (looking for patterns) and code transformation (automated updates for major code changes is a common use case).

Thread chose to build on top of `ast-grep` because it provides a solid foundation for parsing and understanding code. It has a battle-tested parsing engine, sophisticated pattern matching system, and an innovative approach to providing a standardized interface for tree-sitter parsers (implemented with a single line of code as a macro).

`Thread` will be both a library and CLI tool. The CLI tool provides a convenient way to interact with the library.

- The project follows the `mongodb` model -- AGPL 3.0 for the library and cli tool with option to buy commercial license.
- It will form the backbone of a planned cloud service that will provide a web-based interface for interacting with the library and CLI tool. - Importantly, the service layer that handles the cloud service (deployed to CloudFlare Workers using WASM) will be proprietary, but needs to easily integrate with the library and CLI tool. The open-source tool will only have implemented functionality for local use and limited web deployment, but needs to be ready to support the cloud service. There is a wasm build in the open source library, but it will be capped, such as limiting scans to 100 files, requiring an API key for cloud service, etc.

Natively, `ast-grep` consists of a handful of crates -- `cli`, `core`, `language`, `dynamic` (innovatively allowing dynamic language loading), `config` (handles cli configuration with some robust logic for providing custom rulesets in yaml -- for both search and search/replace). There are also a few other crates that aren't relevant here (pyO3 bindings, NAPI bindings, etc.).

## Current Status

We've done very little work on `Thread` itself, instead engaged on getting the ast-grep foundation 'right'. We broke apart the `ast-grep` codebase into a set of crates by functionality. The `ast-grep-core` crate is actually designed to be used as a library, but it leaves out a lot of capability from what the CLI tool provides.

### Incomplete Service Implementations

1. `thread-services`: This is the main skeleton/scaffold for the service-oriented architecture.
2. `ag-service-types`: While most of the crate is the core types from ast-grep, the `lib.rs` file contains a series of types and implementations intended as a service-oriented architecture. It is not complete, and I honestly don't know if it is well thought out, or not. It is a remnant of our initial attempt at implementing a service-oriented architecture.

### Motivations for Breaking up the Ast-Grep Codebase into so many crates

- The overall goal was getting the flexibility to isolate builds to be able to deploy focused services to WASM without pulling in unnecessary dependencies. It's not important for the CLI tool, but it's very important for the cloud service. We chose this course after we initially tried to just use ast-grep as a library, or do minimal vendoring, such as of the language crate, but it became clear we would have to choose either "just use the core and lose the rest of the ast-grep, or bite the bullet and make ast-grep the library we would need it to be to keep all of its functions.
- We also tried just re-exporting `ast-grep-core` with narrower feature gates but it was very messy.

### Ast-Grep Crates

Ultimately, we currently have the following crates. We adopted a naming scheme of `ag-service-*` to make it clear it's not the original `ast-grep` codebase, but rather a service-oriented architecture built on top of it (ast-grep is MIT licensed):

- `ag-service-types`: We moved all type definitions (struct, enum, trait) from the other crates and consolidated them by function into this crate. There are no implementations here, just definitions, and each function is feature-gated so that we can pick and choose what we want to include in a given build. It's kind of big, but it does make the codebase easier to reason about.
  - Separating these out was a lot of work, we should be willing to undo it if it doesn't make sense, but the benefits need to be clear. It did help decouple the type dependencies (when we don't need the implemented traits, it keeps us from pulling in a lot of code).
- `ag-service-ast`: The core ast-grep AST functionality from `ast-grep-core`, but without the tree-sitter dependencies (it's the layer above the tree manipulation logic).
- `ag-service-check-rule`: This was a module in the CLI. It offers validation of rules, which is useful for testing rules before using them on a recurring basis. It won't be as frequent as scanning, but it is a good service to have.
- `ag-service-fix`: This was a module in the CLI. It provides functionality for fixing code based on rules, which is useful for automated updates for major code changes.
- `ag-label`: This was part of the `config` crate in `ast-grep`. It provides functionality for labeling code and is interface agnostic. It has limited use in the library today, only integrating with the rule handling logic, but it could be used more widely.
- `ag-service-ops` - also from the CLI, it implements logical operations logic for tree parsing (Any, All, Or, Not)
- `ag-service-pattern`: This was the core matching logic in ast-grep `core`. It provides/implements the `Pattern` and `NodeMatch` types that support ast-grep's sophisticated pattern matching system.
- `ag-service-rule`: This was the lion's share of the `ast-grep-config` crate. It provides functionality for defining and managing rules, which is a core part of the ast-grep functionality.
- `ag-service-scan`: This was the CLI 'scan' command. It provides functionality for scanning code based on rules. The `lib.rs` interface is actually something we added and may not be needed.
- `ag-service-transform`: This was the `Replacer` module in `ast-grep-core`. It provides functionality for transforming code based on rules, which is a core part of the ast-grep functionality.
- `ag-service-tree-sitter`: This is the actual tree-sitter interface that handles Tree transversal and manipulation.
- `ag-service-utils`: This was a module in the CLI. It provides a set of utilities that are mostly CLI focused, but some of them may be more broadly useful if separated out. Minimally it will help us save time on CLI implementation.
- `thread-threadlang` and `thread-languages`: `thread-languages` is almost exactly the same as `ast-grep-language`, but we needed important components public that weren't exposed in the original crate. Primarily that helped with feature gating each language. `thread-threadlang` provides the `ThreadLang` type, renamed from `SgLang`, which is the main interface for interacting with languages in `ast-grep`. It provides a standardized interface for tree-sitter parsers. We broke this off from partly being in the CLI and partly in the `language` crate. We chose to put these in the main `thread` crates because they are much more core to our vision of Thread.

> NOTE: The workspace does not currently build. Dependencies are a mess from the reorganization, but I don't think it's worth fixing until we've finalized the architecture.

## Ast-Grep As a Service

`Thread` is primarily focused on providing intelligent code context for AI assistants (and humans too), but we saw a lot of potential for using the full ast-grep functionality as a secondary service. (For example, it could provide an interface for AI assistants to replace code). We see it as supporting important parts of our longer-term vision. It is important to understand that `ast-grep` and `Thread`'s needs diverge at about the point where Ast-Grep first parses the code. From there, `Thread` will build a graph of the code, while `ast-grep` will focus on pattern matching and transformation. But each of those services need to be separate and not assumed -- there's no "one" pipeline.

## Questions and Considerations about the Current State of the Codebase

1. Does `ag-service-scan` still make sense on its own?

2. Does `ag-service-fix` and `ag-service-transform` make sense as separate crates? Fix originated with the CLI 'fix' command, while transform came from `ast-grep-core`. There's something similar with `as-service-scan` and `ag-service-pattern`. Scan came from the CLI 'scan' command, while pattern came from `ast-grep-core`.

   - `ag-service-fix` and `ag-service-transform` could potentially be merged into a single crate, as they both deal with modifying the AST in some way. However, if they have distinct use cases or APIs, keeping them separate might make sense.

3. How can we better integrate `ag-service-check_rule` with the rest of the codebase? How can it be separated from CLI and made more reusable?

   - it is a good service insofar as you would want to test rules before using them on a recurring basis. It won't be as frequent as scanning, but it is a good service to have.

4. `ag-service-utils` currently provides a set of utilities that are mostly CLI focused. Some of these may be more broadly useful if separated out.

   - What should stay, what should go, and how should we structure it?
   - While narrower in focus, the CLI utilities are also useful... for the CLI when we get to that point. We could park them in our thread-cli crate.

5. The core `ag-service-types/lib.rs` is a remnant of our initial attempt at implementing a service-oriented architecture. There's also the skeleton of a separate attempt in `thread-services`.

   - We need to probably consolidate these efforts and decide on a single approach.
   - Any services should be able to use all of the ast-grep services with their native types. We should not have to reinvent the wheel for each service.
   - It does make sense to have a common set of interfaces so that we can implement services that can be used across different parts of the codebase.

6. In general, I don't love how hard it is to isolate functionality from things like serialization/deserialization, schema generation, yaml config assumptions, CLI, and other concerns. We don't need those concerns in many of our use cases, and we need to keep binaries small for wasm. Ideally, we should be able to pick and choose core functionality without pulling anything we don't strictly need.
   - This is all of course because we pulled it out of a CLI tool. So the codebase still assumes a lot of CLI-related functionality.
      - We need it to not assume:
        - Output requires serialization
        - Input requires deserialization
        - Configuration requires yaml (or any file) input
        - Discovery requires a filesystem
        - CLI requires a terminal (usually true, but not always)
        - That input or output will come from any specific format or source (ie, JSON, YAML, etc.)
        - That multi-threading is available (but use it when we can)
      - The FastMap/FastSet types help with this, but we really need an execution layer that can be used across different contexts without assuming a lot of things.
      - I think the service-oriented architecture is probably the right way to go, but our current attempts are very rough and not well thought out.

7. `thread-threadlang` inherited some modules from `ast-grep-cli` that we weren't sure where to put. They handle some important functionality, like parsing embedded code (like javascript within HTML). This probably isn't the right place for them, or at least not all of them. We should probably move them to a more appropriate place.
