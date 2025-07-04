// crates/thread-parser/build.rs
use std::path::PathBuf;
use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=tree-sitter-grammars/");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Generate type-sitter bindings
    #[cfg(feature = "rust")]
    {
        let rust_dir = out_dir.join("rust");
        std::fs::create_dir_all(&rust_dir).unwrap();

        type_sitter_gen::generate_nodes(
            &PathBuf::from("tree-sitter-grammars/tree-sitter-rust"),
            &rust_dir.join("nodes.rs"),
        );

        println!("cargo:rustc-env=RUST_NODES_PATH={}", rust_dir.join("nodes.rs").display());
    }

    #[cfg(feature = "javascript")]
    {
        let js_dir = out_dir.join("javascript");
        std::fs::create_dir_all(&js_dir).unwrap();

        type_sitter_gen::generate_nodes(
            &PathBuf::from("tree-sitter-grammars/tree-sitter-javascript"),
            &js_dir.join("nodes.rs"),
        );

        println!("cargo:rustc-env=JS_NODES_PATH={}", js_dir.join("nodes.rs").display());
    }
}
