<!--
SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Thread Core

This crate provides the core types and traits for Thread.

We'll mostly discuss these features so you can understand how to use them.

## Features

### Utility Features

Thread Core provides several utility-oriented features that provide types that improve performance in certain settings, or provide additional functionality that is not strictly necessary for the core functionality of Thread. These features are:

- `fastmap`: Provides a `FastMap` type that either provides a `DashMap`  (and `DashSet`) for blazing concurrent access, or a std `HashMap` (and `HashSet`) for single-threaded access. This is useful for caching and storing data that needs to be accessed quickly and concurrently. For it to provide a `Dashmap`, the `dashmap` feature must *also* be enabled. Both are enabled by default.
  - Note: Regardless of flags you pass, the `FastMap` will always revert to a `HashMap` and `HashSet` if the `wasm-single-thread` feature is enabled. This saves on the size of the binary, and is primarily intended for Cloudflare Workers and similar cloud WASM environments that do not support multi-threading. You *can* use `DashMap` in other WASM environments like browsers or `WASI`.
- `inline` and `dash-inline`: Enables more aggressive inlining of functions and methods, which can improve performance in some cases at the expense of longer compile times. The `dash-inline` feature enables inlining for `DashMap` and `DashSet` methods, while `inline` enables it for both DashMap and `string-interner`. `inline` is not default, but we do use it in our prebuilt release binaries (github releases).
