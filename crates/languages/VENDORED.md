# Thread Languages: A Fork of Ast-Grep-Language

The `thread-language` crate is a vendored-fork of `ast-grep-language`. We've made changes to the structure, dependencies, and functionality to better suit our needs.

Like the rest of the [Ast-Grep][AG] project, it was a fantastic foundation to build on.

## Forked at Ast-Grep Version: 0.38.7

## Why we Forked

`ast-grep-language` was a great starting point -- particularly its innovative macro system and framework for adding languages while giving them a common interface with Rust typing. We needed to change a couple of things:

## What we Changed

- We needed to be able to add more languages, and there was no way to do this with the existing structure (besides the `ast-grep-dynamic` crate which is great but not what we needed). Since we're building a library to support different uses, and aren't constrained to a CLI binary in all cases, we needed a more flexible approach.
- We needed a flatter structure to support different deployment options and to minimize binary size for Knitli's cloud microservice deployment.
- We wanted to separate the `SgLang` type into its own crate for slimmer deployments for some use cases. This is now the `threadlang` crate.

As it stands, we can selectively take upstream changes from `ast-grep-language` for most of the crate while keeping our modifications intact. We can, and will, also submit bug fixes upstream when we identify issues with shared code.

## Ast-Grep is awesome

Ast-Grep was created by and copyrighted to Herrington Darkholme under the MIT license. We kept the same licensing on our fork to avoid making things more complicated for anyone who wants to build from either.

[Ast-Grep][AG] provides a powerful framework for parsing, searching, targeting, and replacing code using [Tree-sitter's][ts] Concrete Syntax Trees[^1]. When we looked to build our tool with Tree-sitter, it quickly became clear that Ast-Grep already solved the more painful parts of that, and much more elegantly than we could have. Building on top of Ast-Grep freed us to focus on core functionality, and leave the hard work of wrangling syntax trees to its maintainers.

[^1]: For reasons none of us really understand, Tree-sitter, Ast-Grep, Thread, and the rest of the Tree-sitter ecosystem talk about the trees we build and analyze as 'AST', or Abstract Syntax Trees, when we all know they're 'CSTs'. 🤷‍♂️ I think maybe it's like Fight Club -- the first rule of Tree-sitter Club is "pretend it's an AST."

[AG]: <https://github.com/ast-grep/ast-grep> "Check out Ast-Grep, it's awesome"
[ts]: <https://github.com/tree-sitter/tree-sitter> "Check out Tree-sitter, it's also awesome"
