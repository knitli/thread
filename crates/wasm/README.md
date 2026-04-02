<!--
SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# thread-wasm

WebAssembly bindings for Thread. Deploy Thread's AST analysis capabilities to the web, edge
runtimes, and Cloudflare Workers.

## Overview

`thread-wasm` compiles Thread to WebAssembly via [`wasm-bindgen`](https://github.com/rustwasm/wasm-bindgen),
enabling browser and edge deployments of the Thread code analysis engine.

## Deployment Targets

| Target | Feature | Use case |
|--------|---------|----------|
| **Cloudflare Workers** | `worker` (default) | Serverless edge analysis, single-threaded |
| **Browser** | `browser` | In-browser analysis with optional multi-threading |

## Building

```bash
# Build for Cloudflare Workers (default, single-threaded)
cargo run -p xtask build-wasm --release

# Build for browsers with multi-threading support
cargo run -p xtask build-wasm --multi-threading --release

# Development build (faster, unoptimized)
cargo run -p xtask build-wasm
```

The build outputs a `pkg/` directory containing the WASM binary and JavaScript glue code, and
`dist/thread-wasm.optimized.wasm` for the final optimized binary.

## Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `worker` | Single-threaded Cloudflare Workers target | ✅ |
| `browser` | Browser target with serialization and optional threading | — |
| `multi-threading` | Enable Rayon-based multi-threading (browser only) | — |
| `serialization` | `serde`-based serialization for JS interop | — |
| `panic-hook` | Better panic messages via `console.error` | — |

## Edge Deployment (Cloudflare Workers)

Thread's WASM build integrates with the Cloudflare Workers runtime for globally distributed
code analysis:

```bash
# Build and deploy
cargo run -p xtask build-wasm --release
wrangler deploy
```

See [Edge Deployment Guide](../../docs/deployment/EDGE_DEPLOYMENT.md) for the full setup.

## Related Crates

- [`thread`](../thread) — Unified entry point crate (uses `worker` feature for WASM builds)
- [`thread-language`](../language) — Language parsers (WASM-compatible via `worker` feature set used by `thread-wasm`)
- [`xtask`](../../xtask) — Custom build tasks for WASM compilation and optimization

## License

AGPL-3.0-or-later

