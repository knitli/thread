// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use pico_args::Arguments;
use std::path::Path;
use std::process::{Command, exit};

const CRATE_PATH: &str = "crates/thread-wasm";
const PKG_PATH: &str = "crates/thread-wasm/pkg";
const DIST_PATH: &str = "dist/thread-wasm.optimized.wasm";

const HELP: &str = r"
xtask - Build thread-wasm WASM binary

Usage:
  xtask build-wasm [--multi-threading] [--release] [--profiling]

Defaults:
  no multi-threading (intended for cloudflare workers)
  dev build (no optimizations)

Options:
    --multi-threading    Enable multi-threading features in the WASM build (for browser deployments)
    --release            Build in release mode with optimizations
    --profiling          Build with profiling enabled (no optimizations)
    --help, -h          Show this help message
";

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
/// Represents the build mode for the WASM binary.
enum BuildMode {
    Release,
    #[default]
    Dev,
    Profiling,
}

impl BuildMode {
    const fn as_wasm_pack_flag(self) -> Option<&'static str> {
        match self {
            Self::Release => Some("--release"),
            _ => None,
        }
    }
    fn opt_options(self) -> Vec<&'static str> {
        match self {
            Self::Release => vec![
                "-O4",
                "--enable-bulk-memory",
                "--enable-sign-ext",
                "--strip-debug",
            ],
            Self::Dev => vec!["-O", "--symbolmap", "--safe-heap"],
            Self::Profiling => vec!["-O", "--enable-bulk-memory", "--enable-sign-ext"],
        }
    }
}

fn main() {
    let mut args = Arguments::from_env();
    if args.contains("--help") || args.contains("-h") {
        println!("{HELP}");
        exit(0);
    }
    let subcmd = args.subcommand().unwrap_or(Some("build-wasm".to_string()));

    match subcmd.as_deref() {
        Some("build-wasm") => {
            let multi = args.contains("--multi-threading");
            let mode = if args.contains("--release") {
                BuildMode::Release
            } else if args.contains("--profiling") {
                BuildMode::Profiling
            } else {
                BuildMode::Dev
            };
            build_wasm(mode, multi);
        }
        Some(cmd) => {
            eprintln!("Unknown subcommand: {cmd}");
            exit(1);
        }
        None => {
            println!("{HELP}");
            exit(1);
        }
    }
}

fn build_wasm(mode: BuildMode, multi: bool) {
    // wasm-pack build [crate-path] --target web [--release] [--features multi-threading]
    let mut wasm_pack = Command::new("wasm-pack");
    wasm_pack.args(["build", CRATE_PATH, "--target", "web"]);
    if let Some(flag) = mode.as_wasm_pack_flag() {
        wasm_pack.arg(flag);
        wasm_pack.args(["--features", "inline"]);
    }
    if multi {
        // we already have a --features flag if we're releasing
        if mode == BuildMode::Release {
            wasm_pack.arg("multi-threading");
        } else {
            wasm_pack.args(["--features", "multi-threading"]);
        }
    }
    run_or_die(wasm_pack, "wasm-pack build");

    // Locate the pkg/ .wasm file (there may be several, pick the *_bg.wasm as main artifact)
    let pkg_dir = Path::new(PKG_PATH);
    let bg_wasm = std::fs::read_dir(pkg_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .find(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .ends_with("_bg.wasm")
        })
        .expect("No *_bg.wasm file in pkg dir");

    std::fs::create_dir_all("dist").unwrap();

    let mut wasm_opt = Command::new("wasm-opt");
    wasm_opt.arg(&bg_wasm);
    wasm_opt.args(mode.opt_options());
    wasm_opt.args([
        "--enable-multivalue",
        "--vacuum",
        "--enable-tail-call",
        "--enable-reference-types",
        "--enabled-non-trapping-float-to-int",
        "--enable-relaxed-simd",
    ]);
    if multi {
        wasm_opt.args(["--enable-threads", "--disable-multi-memories"]);
    } else {
        wasm_opt.args(["--disable-threads", "--enable-multi-memories"]);
    }
    wasm_opt.args(["-o", DIST_PATH]);

    run_or_die(wasm_opt, "wasm-opt");

    println!("Built optimized wasm to {DIST_PATH:?}");
}

fn run_or_die(mut cmd: Command, label: &str) {
    let status = cmd.status().unwrap();
    if !status.success() {
        eprintln!("{label} failed");
        exit(1);
    }
}
