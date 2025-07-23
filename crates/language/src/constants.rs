// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::SupportLang;

pub const ALL_SUPPORTED_LANGS: [&'static str; 23] = [
    "bash",
    "c",
    "cpp",
    "csharp",
    "css",
    "elixir",
    "go",
    "haskell",
    "html",
    "java",
    "javascript",
    "json",
    "kotlin",
    "lua",
    "php",
    "python",
    "rust",
    "ruby",
    "scala",
    "swift",
    "typescript",
    "tsx",
    "yaml",
];

#[cfg(any(feature = "bash", feature = "all-parsers"))]
pub const BASH_EXTS: [&'static str; 19] = [
    "bash",
    "bats",
    "sh",
    ".bashrc",
    "bash_aliases",
    "bats",
    "cgi",
    "command",
    "env",
    "fcgi",
    "ksh",
    "tmux",
    "tool",
    "zsh",
    "bash_logout",
    "bash_profile",
    "profile",
    "login",
    "logout",
];

// C++ is more popular, at least according to GitHub's language stats,
// so we prioritize it over C for the 'h' filetype when both are enabled.
cfg_if::cfg_if! {
    if #[cfg(all(feature = "c", not(feature = "cpp")))] {
        pub const C_EXTS: [&'static str; 2] = ["c", "h"];
    } else if #[cfg(any(feature = "c", feature = "all-parsers"))] {
        pub const C_EXTS: [&'static str; 1] = ["c"];
    }
}

/// C++ specific extensions; we consider cuda c++ for our purposes
#[cfg(any(feature = "cpp", feature = "all-parsers"))]
pub const CPP_EXTS: [&'static str; 11] = [
    "cpp", "cc", "cxx", "hxx", "c++", "hh", "cxx", "cu", "ino", "h", "cu",
];

#[cfg(any(feature = "csharp", feature = "all-parsers"))]
pub const CSHARP_EXTS: [&'static str; 2] = ["cs", "csx"];

#[cfg(any(feature = "css", feature = "all-parsers", feature = "css-napi", feature = "napi-compatible"))]
pub const CSS_EXTS: [&'static str; 1] = ["css"];

#[cfg(any(feature = "elixir", feature = "all-parsers"))]
pub const ELIXIR_EXTS: [&'static str; 2] = ["ex", "exs"];

#[cfg(any(feature = "go", feature = "all-parsers"))]
pub const GO_EXTS: [&'static str; 1] = ["go"];

#[cfg(feature = "haskell")]
pub const HASKELL_EXTS: [&'static str; 2] = ["hs", "lhs"];

#[cfg(any(
    feature = "html",
    feature = "all-parsers",
    feature = "html-napi",
    feature = "napi-compatible"
))]
pub const HTML_EXTS: [&'static str; 4] = ["html", "htm", "xhtml", "shtml"];

#[cfg(any(feature = "java", feature = "all-parsers"))]
pub const JAVA_EXTS: [&'static str; 1] = ["java"];

#[cfg(any(
    feature = "javascript",
    feature = "all-parsers",
    feature = "javascript-napi",
    feature = "napi-compatible"
))]
pub const JAVASCRIPT_EXTS: [&'static str; 5] = ["js", "mjs", "cjs", "jsx", "snap"];

#[cfg(any(feature = "json", feature = "all-parsers"))]
pub const JSON_EXTS: [&'static str; 3] = ["json", "json5", "jsonc"];

#[cfg(any(feature = "kotlin", feature = "all-parsers"))]
pub const KOTLIN_EXTS: [&'static str; 3] = ["kt", "kts", "ktm"];

#[cfg(any(feature = "lua", feature = "all-parsers"))]
pub const LUA_EXTS: [&'static str; 1] = ["lua"];

#[cfg(any(feature = "php", feature = "all-parsers"))]
pub const PHP_EXTS: [&'static str; 2] = ["php", "phtml"];

#[cfg(any(feature = "python", feature = "all-parsers"))]
pub const PYTHON_EXTS: [&'static str; 4] = ["py", "py3", "pyi", "bzl"];

#[cfg(any(feature = "ruby", feature = "all-parsers"))]
pub const RUBY_EXTS: [&'static str; 4] = ["rb", "rbw", "rake", "gemspec"];

#[cfg(any(feature = "rust", feature = "all-parsers"))]
pub const RUST_EXTS: [&'static str; 1] = ["rs"];

#[cfg(any(feature = "scala", feature = "all-parsers"))]
pub const SCALA_EXTS: [&'static str; 4] = ["scala", "sc", "scm", "sbt"];

#[cfg(any(feature = "swift", feature = "all-parsers"))]
pub const SWIFT_EXTS: [&'static str; 2] = ["swift", "xctest"];

#[cfg(any(
    feature = "typescript",
    feature = "all-parsers",
    feature = "typescript-napi",
    feature = "napi-compatible"
))]
pub const TYPESCRIPT_EXTS: [&'static str; 3] = ["ts", "cts", "mts"];

#[cfg(any(feature = "tsx", feature = "all-parsers", feature = "tsx-napi", feature = "napi-compatible"))]
pub const TSX_EXTS: [&'static str; 1] = ["tsx"];

#[cfg(any(feature = "yaml", feature = "all-parsers"))]
pub const YAML_EXTS: [&'static str; 2] = ["yaml", "yml"];

cfg_if::cfg_if!(
    if #[cfg(
            not(
                any(
                    feature = "bash", feature = "c", feature = "cpp",
                    feature = "csharp", feature = "css", feature = "elixir",
                    feature = "go", feature = "haskell", feature = "html",
                    feature = "java", feature = "javascript", feature = "json",
                    feature = "kotlin", feature = "lua", feature = "php",
                    feature = "python", feature = "ruby", feature = "rust",
                    feature = "scala", feature = "swift", feature = "tsx",
                    feature = "typescript", feature = "yaml"
                    )
                )
    )] {
        pub const ENABLED_LANGS: &'static [&'static crate::SupportLang; 1] = &[crate::SupportLang::NoEnabledLangs];
    } else {
    pub const ENABLED_LANGS: &'static [&'static SupportLang] = &{
        // Count total enabled languages
        use crate::SupportLang::*;
        const fn count_enabled_langs() -> usize {

            let mut count = 0;

            #[cfg(any(feature = "bash", feature = "all-parsers"))]
            { count += 1; }
            #[cfg(any(feature = "c", feature = "all-parsers"))]
            { count += 1; }
            #[cfg(any(feature = "cpp", feature = "all-parsers"))]
            { count += 1; }
            #[cfg(any(feature = "csharp", feature = "all-parsers"))]
            { count += 1; }
            #[cfg(any(feature = "css", feature = "all-parsers", feature = "css-napi", feature = "napi-compatible"))]
            { count += 1; }
            #[cfg(any(feature = "elixir", feature = "all-parsers"))]
            { count += 1; }
            #[cfg(any(feature = "go", feature = "all-parsers"))]
            { count += 1; }
            #[cfg(feature = "haskell")]
            { count += 1; }
            #[cfg(any(feature = "html", feature = "all-parsers", feature = "html-napi", feature = "napi-compatible"))]
            { count += 1; }
            #[cfg(any(feature = "java", feature = "all-parsers"))]
            { count += 1; }
            #[cfg(any(feature = "javascript", feature = "all-parsers", feature = "javascript-napi", feature = "napi-compatible"))]
            { count += 1; }
            #[cfg(any(feature = "json", feature = "all-parsers"))]
            { count += 1; }
            #[cfg(any(feature = "kotlin", feature = "all-parsers"))]
            { count += 1; }
            #[cfg(any(feature = "lua", feature = "all-parsers"))]
            { count += 1; }
            #[cfg(any(feature = "php", feature = "all-parsers"))]
            { count += 1; }
            #[cfg(any(feature = "python", feature = "all-parsers"))]
            { count += 1; }
            #[cfg(any(feature = "ruby", feature = "all-parsers"))]
            { count += 1; }
            #[cfg(any(feature = "rust", feature = "all-parsers"))]
            { count += 1; }
            #[cfg(any(feature = "scala", feature = "all-parsers"))]
            { count += 1; }
            #[cfg(any(feature = "swift", feature = "all-parsers"))]
            { count += 1; }
            #[cfg(any(feature = "typescript", feature = "all-parsers", feature = "typescript-napi", feature = "napi-compatible"))]
            { count += 1; }
            #[cfg(any(feature = "tsx", feature = "all-parsers", feature = "tsx-napi", feature = "napi-compatible"))]
            { count += 1; }
            #[cfg(any(feature = "yaml", feature = "all-parsers"))]
            { count += 1; }

            count
        }

        // Build the enabled languages array at compile time
        const fn build_enabled_langs_array() -> [&'static SupportLang; count_enabled_langs()] {

            let mut result: [&'static SupportLang; count_enabled_langs()] =
                [&SupportLang::all_langs()[0]; count_enabled_langs()];

                let mut index: usize = 0;

                #[cfg(any(feature = "bash", feature = "all-parsers"))] {
                    result[index] = &Bash;
                    index += 1;
                }
                #[cfg(any(feature = "c", feature = "all-parsers"))] {
                    result[index] = &C;
                    index += 1;
                }
                #[cfg(any(feature = "cpp", feature = "all-parsers"))] {
                    result[index] = &Cpp;
                    index += 1;
                }
                #[cfg(any(feature = "csharp", feature = "all-parsers"))] {
                    result[index] = &CSharp;
                    index += 1;
                }
                #[cfg(any(feature = "css", feature = "all-parsers", feature = "css-napi", feature = "napi-compatible"))] {
                    result[index] = &Css;
                    index += 1;
                }
                #[cfg(any(feature = "elixir", feature = "all-parsers"))] {
                    result[index] = &Elixir;
                    index += 1;
                }
                #[cfg(any(feature = "go", feature = "all-parsers"))] {
                    result[index] = &Go;
                    index += 1;
                }
                #[cfg(feature = "haskell")] {
                    result[index] = &Haskell;
                    index += 1;
                }
                #[cfg(any(feature = "html", feature = "all-parsers", feature = "html-napi", feature = "napi-compatible"))] {
                    result[index] = &Html;
                    index += 1;
                }
                #[cfg(any(feature = "java", feature = "all-parsers"))] {
                    result[index] = &Java;
                    index += 1;
                }
                #[cfg(any(feature = "javascript", feature = "all-parsers", feature = "javascript-napi", feature = "napi-compatible"))] {
                    result[index] = &JavaScript;
                    index += 1;
                }
                #[cfg(any(feature = "json", feature = "all-parsers"))] {
                    result[index] = &Json;
                    index += 1;
                }
                #[cfg(any(feature = "kotlin", feature = "all-parsers"))] {
                    result[index] = &Kotlin;
                    index += 1;
                }
                #[cfg(any(feature = "lua", feature = "all-parsers"))] {
                    result[index] = &Lua;
                    index += 1;
                }
                #[cfg(any(feature = "php", feature = "all-parsers"))] {
                    result[index] = &Php;
                    index += 1;
                }
                #[cfg(any(feature = "python", feature = "all-parsers"))] {
                    result[index] = &Python;
                    index += 1;
                }
                #[cfg(any(feature = "ruby", feature = "all-parsers"))] {
                    result[index] = &Ruby;
                    index += 1;
                }
                #[cfg(any(feature = "rust", feature = "all-parsers"))] {
                    result[index] = &Rust;
                    index += 1;
                }
                #[cfg(any(feature = "scala", feature = "all-parsers"))] {
                    result[index] = &Scala;
                    index += 1;
                }
                #[cfg(any(feature = "swift", feature = "all-parsers"))] {
                    result[index] = &Swift;
                    index += 1;
                }
                #[cfg(any(feature = "typescript", feature = "all-parsers", feature = "typescript-napi", feature = "napi-compatible"))] {
                    result[index] = &TypeScript;
                    index += 1;
                }
                #[cfg(any(feature = "tsx", feature = "all-parsers", feature = "tsx-napi", feature = "napi-compatible"))] {
                    result[index] = &Tsx;
                    index += 1;
                }
                #[cfg(any(feature = "yaml", feature = "all-parsers"))] {
                    result[index] = &Yaml;
                    index += 1;
                }
                let _ = index; // Mark index as used to avoid unused assignment warning
                result
            }
        build_enabled_langs_array()
    };
});

cfg_if::cfg_if!(
    if #[cfg(
            not(
                any(feature = "all-parsers", feature = "napi-compatible", feature = "css-napi", feature = "html-napi", feature = "javascript-napi", feature = "typescript-napi", feature = "tsx-napi",
                    feature = "bash", feature = "c", feature = "cpp",
                    feature = "csharp", feature = "css", feature = "elixir",
                    feature = "go", feature = "haskell", feature = "html",
                    feature = "java", feature = "javascript", feature = "json",
                    feature = "kotlin", feature = "lua", feature = "php",
                    feature = "python", feature = "ruby", feature = "rust",
                    feature = "scala", feature = "swift", feature = "tsx",
                    feature = "typescript", feature = "yaml"
                    )
                )
            )] {
                pub const EXTENSIONS: &'static [&'static str; 0] = &[]
            } else {
            pub const EXTENSIONS: &'static [&'static str] = &{
                // Count total extensions needed
                const fn count_total_extensions() -> usize {
                    let mut count = 0;

                    #[cfg(any(feature = "bash", feature = "all-parsers"))]
                    { count += BASH_EXTS.len(); }

                    #[cfg(all(feature = "c", not(feature = "cpp")))]
                    { count += C_EXTS.len(); }
                    #[cfg(all(feature = "c", feature = "cpp"))]
                    { count += C_EXTS.len(); }

                    #[cfg(any(feature = "cpp", feature = "all-parsers"))]
                    { count += CPP_EXTS.len(); }

                    #[cfg(any(feature = "csharp", feature = "all-parsers"))]
                    { count += CSHARP_EXTS.len(); }

                    #[cfg(any(feature = "css", feature = "all-parsers", feature = "css-napi", feature = "napi-compatible"))]
                    { count += CSS_EXTS.len(); }

                    #[cfg(any(feature = "elixir", feature = "all-parsers"))]
                    { count += ELIXIR_EXTS.len(); }

                    #[cfg(any(feature = "go", feature = "all-parsers"))]
                    { count += GO_EXTS.len(); }

                    #[cfg(feature = "haskell")]
                    { count += HASKELL_EXTS.len(); }

                    #[cfg(any(feature = "html", feature = "all-parsers", feature = "html-napi", feature = "napi-compatible"))]
                    { count += HTML_EXTS.len(); }

                    #[cfg(any(feature = "java", feature = "all-parsers"))]
                    { count += JAVA_EXTS.len(); }

                    #[cfg(any(feature = "javascript", feature = "all-parsers", feature = "javascript-napi", feature = "napi-compatible"))]
                    { count += JAVASCRIPT_EXTS.len(); }

                    #[cfg(any(feature = "json", feature = "all-parsers"))]
                    { count += JSON_EXTS.len(); }

                    #[cfg(any(feature = "kotlin", feature = "all-parsers"))]
                    { count += KOTLIN_EXTS.len(); }

                    #[cfg(any(feature = "lua", feature = "all-parsers"))]
                    { count += LUA_EXTS.len(); }

                    #[cfg(any(feature = "php", feature = "all-parsers"))]
                    { count += PHP_EXTS.len(); }

                    #[cfg(any(feature = "python", feature = "all-parsers"))]
                    { count += PYTHON_EXTS.len(); }

                    #[cfg(any(feature = "ruby", feature = "all-parsers"))]
                    { count += RUBY_EXTS.len(); }

                    #[cfg(any(feature = "rust", feature = "all-parsers"))]
                    { count += RUST_EXTS.len(); }

                    #[cfg(any(feature = "scala", feature = "all-parsers"))]
                    { count += SCALA_EXTS.len(); }

                    #[cfg(any(feature = "swift", feature = "all-parsers"))]
                    { count += SWIFT_EXTS.len(); }

                    #[cfg(any(feature = "typescript", feature = "all-parsers", feature = "typescript-napi", feature = "napi-compatible"))]
                    { count += TYPESCRIPT_EXTS.len(); }

                    #[cfg(any(feature = "tsx", feature = "all-parsers", feature = "tsx-napi", feature = "napi-compatible"))]
                    { count += TSX_EXTS.len(); }

                    #[cfg(any(feature = "yaml", feature = "all-parsers"))]
                    { count += YAML_EXTS.len(); }

                    count
                }

                // Build the flattened array at compile time
                const fn build_extensions_array() -> [&'static str; count_total_extensions()] {
                    let mut result = [""; count_total_extensions()];
                    let mut index = 0;

                    #[cfg(any(feature = "bash", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < BASH_EXTS.len() {
                            result[index] = BASH_EXTS[i];
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(all(feature = "c", not(feature = "cpp")))]
                    {
                        let mut i = 0;
                        while i < C_EXTS.len() {
                            result[index] = C_EXTS[i];
                            index += 1;
                            i += 1;
                        }
                    }
                    #[cfg(all(feature = "c", feature = "cpp"))]
                    {
                        let mut i = 0;
                        while i < C_EXTS.len() {
                            result[index] = C_EXTS[i];
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "cpp", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < CPP_EXTS.len() {
                            result[index] = CPP_EXTS[i];
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "csharp", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < CSHARP_EXTS.len() {
                            result[index] = CSHARP_EXTS[i];
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "css", feature = "all-parsers", feature = "css-napi", feature = "napi-compatible"))]
                    {
                        let mut i = 0;
                        while i < CSS_EXTS.len() {
                            result[index] = CSS_EXTS[i];
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "elixir", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < ELIXIR_EXTS.len() {
                            result[index] = ELIXIR_EXTS[i];
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "go", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < GO_EXTS.len() {
                            result[index] = GO_EXTS[i];
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(feature = "haskell")]
                    {
                        let mut i = 0;
                        while i < HASKELL_EXTS.len() {
                            result[index] = HASKELL_EXTS[i];
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "html", feature = "all-parsers", feature = "html-napi", feature = "napi-compatible"))]
                    {
                        let mut i = 0;
                        while i < HTML_EXTS.len() {
                            result[index] = HTML_EXTS[i];
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "java", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < JAVA_EXTS.len() {
                            result[index] = JAVA_EXTS[i];
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "javascript", feature = "all-parsers", feature = "javascript-napi", feature = "napi-compatible"))]
                    {
                        let mut i = 0;
                        while i < JAVASCRIPT_EXTS.len() {
                            result[index] = JAVASCRIPT_EXTS[i];
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "json", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < JSON_EXTS.len() {
                            result[index] = JSON_EXTS[i];
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "kotlin", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < KOTLIN_EXTS.len() {
                            result[index] = KOTLIN_EXTS[i];
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "lua", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < LUA_EXTS.len() {
                            result[index] = LUA_EXTS[i];
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "php", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < PHP_EXTS.len() {
                            result[index] = PHP_EXTS[i];
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "python", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < PYTHON_EXTS.len() {
                            result[index] = PYTHON_EXTS[i];
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "ruby", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < RUBY_EXTS.len() {
                            result[index] = RUBY_EXTS[i];
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "rust", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < RUST_EXTS.len() {
                            result[index] = RUST_EXTS[i];
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "scala", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < SCALA_EXTS.len() {
                            result[index] = SCALA_EXTS[i];
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "swift", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < SWIFT_EXTS.len() {
                            result[index] = SWIFT_EXTS[i];
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "typescript", feature = "all-parsers", feature = "typescript-napi", feature = "napi-compatible"))]
                    {
                        let mut i = 0;
                        while i < TYPESCRIPT_EXTS.len() {
                            result[index] = TYPESCRIPT_EXTS[i];
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "tsx", feature = "all-parsers", feature = "tsx-napi", feature = "napi-compatible"))]
                    {
                        let mut i = 0;
                        while i < TSX_EXTS.len() {
                            result[index] = TSX_EXTS[i];
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "yaml", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < YAML_EXTS.len() {
                            result[index] = YAML_EXTS[i];
                            index += 1;
                            i += 1;
                        }
                    }

                    let _ = index; // Mark index as used to avoid unused assignment warning
                    result
                }

                build_extensions_array()
            };

            }
);

// Generate the flattened extensions array at compile time using const evaluation

// Language lookup array for direct O(1) access from Aho-Corasick match indices.
// Each language is repeated for each of its extensions in the same order as EXTENSIONS.
// This allows us to directly map from a match index to the corresponding SupportLang.
cfg_if::cfg_if!(
        if #[cfg(
            not(
                any(
                    feature = "bash", feature = "c", feature = "cpp",
                    feature = "csharp", feature = "css", feature = "elixir",
                    feature = "go", feature = "haskell", feature = "html",
                    feature = "java", feature = "javascript", feature = "json",
                    feature = "kotlin", feature = "lua", feature = "php",
                    feature = "python", feature = "ruby", feature = "rust",
                    feature = "scala", feature = "swift", feature = "tsx",
                    feature = "typescript", feature = "yaml", feature = "napi-compatible", feature = "css-napi", feature = "html-napi", feature = "javascript-napi", feature = "tsx-napi", feature = "typescript-napi"
                    )
                )
            )] {
                pub const EXTENSION_TO_LANG: &[SupportLang; 1] = &[crate::SupportLang::NoEnabledLangs]
            } else {
            pub const EXTENSION_TO_LANG: &[SupportLang] = &{
                use crate::SupportLang;

                // Count total extensions needed (same as EXTENSIONS array)
                const fn count_total_extensions() -> usize {
                    let mut count = 0;

                    #[cfg(any(feature = "bash", feature = "all-parsers"))]
                    { count += BASH_EXTS.len(); }

                    #[cfg(any(feature = "c", feature = "all-parsers"))]
                    { count += C_EXTS.len(); }

                    #[cfg(any(feature = "cpp", feature = "all-parsers"))]
                    { count += CPP_EXTS.len(); }

                    #[cfg(any(feature = "csharp", feature = "all-parsers"))]
                    { count += CSHARP_EXTS.len(); }

                    #[cfg(any(feature = "css", feature = "all-parsers", feature = "css-napi", feature = "napi-compatible"))]
                    { count += CSS_EXTS.len(); }

                    #[cfg(any(feature = "elixir", feature = "all-parsers"))]
                    { count += ELIXIR_EXTS.len(); }

                    #[cfg(any(feature = "go", feature = "all-parsers"))]
                    { count += GO_EXTS.len(); }

                    #[cfg(feature = "haskell")]
                    { count += HASKELL_EXTS.len(); }

                    #[cfg(any(feature = "html", feature = "all-parsers", feature = "html-napi", feature = "napi-compatible"))]
                    { count += HTML_EXTS.len(); }

                    #[cfg(any(feature = "java", feature = "all-parsers"))]
                    { count += JAVA_EXTS.len(); }

                    #[cfg(any(feature = "javascript", feature = "all-parsers", feature = "javascript-napi", feature = "napi-compatible"))]
                    { count += JAVASCRIPT_EXTS.len(); }

                    #[cfg(any(feature = "json", feature = "all-parsers"))]
                    { count += JSON_EXTS.len(); }

                    #[cfg(any(feature = "kotlin", feature = "all-parsers"))]
                    { count += KOTLIN_EXTS.len(); }

                    #[cfg(any(feature = "lua", feature = "all-parsers"))]
                    { count += LUA_EXTS.len(); }

                    #[cfg(any(feature = "php", feature = "all-parsers"))]
                    { count += PHP_EXTS.len(); }

                    #[cfg(any(feature = "python", feature = "all-parsers"))]
                    { count += PYTHON_EXTS.len(); }

                    #[cfg(any(feature = "ruby", feature = "all-parsers"))]
                    { count += RUBY_EXTS.len(); }

                    #[cfg(any(feature = "rust", feature = "all-parsers"))]
                    { count += RUST_EXTS.len(); }

                    #[cfg(any(feature = "scala", feature = "all-parsers"))]
                    { count += SCALA_EXTS.len(); }

                    #[cfg(any(feature = "swift", feature = "all-parsers"))]
                    { count += SWIFT_EXTS.len(); }

                    #[cfg(any(feature = "typescript", feature = "all-parsers", feature = "typescript-napi", feature = "napi-compatible"))]
                    { count += TYPESCRIPT_EXTS.len(); }

                    #[cfg(any(feature = "tsx", feature = "all-parsers", feature = "tsx-napi", feature = "napi-compatible"))]
                    { count += TSX_EXTS.len(); }

                    #[cfg(any(feature = "yaml", feature = "all-parsers"))]
                    { count += YAML_EXTS.len(); }

                    count
                }

                // Build the language lookup array at compile time
                const fn build_extension_to_lang_array() -> [SupportLang; count_total_extensions()] {
                    let mut result = [SupportLang::all_langs()[0]; count_total_extensions()];
                    let mut index = 0;

                    #[cfg(any(feature = "bash", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < BASH_EXTS.len() {
                            result[index] = SupportLang::Bash;
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "c", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < C_EXTS.len() {
                            result[index] = SupportLang::C;
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "cpp", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < CPP_EXTS.len() {
                            result[index] = SupportLang::Cpp;
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "csharp", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < CSHARP_EXTS.len() {
                            result[index] = SupportLang::CSharp;
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "css", feature = "all-parsers", feature = "css-napi", feature = "napi-compatible"))]
                    {
                        let mut i = 0;
                        while i < CSS_EXTS.len() {
                            result[index] = SupportLang::Css;
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "elixir", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < ELIXIR_EXTS.len() {
                            result[index] = SupportLang::Elixir;
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "go", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < GO_EXTS.len() {
                            result[index] = SupportLang::Go;
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(feature = "haskell")]
                    {
                        let mut i = 0;
                        while i < HASKELL_EXTS.len() {
                            result[index] = SupportLang::Haskell;
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "html", feature = "all-parsers", feature = "html-napi", feature = "napi-compatible"))]
                    {
                        let mut i = 0;
                        while i < HTML_EXTS.len() {
                            result[index] = SupportLang::Html;
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "java", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < JAVA_EXTS.len() {
                            result[index] = SupportLang::Java;
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "javascript", feature = "all-parsers", feature = "javascript-napi", feature = "napi-compatible"))]
                    {
                        let mut i = 0;
                        while i < JAVASCRIPT_EXTS.len() {
                            result[index] = SupportLang::JavaScript;
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "json", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < JSON_EXTS.len() {
                            result[index] = SupportLang::Json;
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "kotlin", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < KOTLIN_EXTS.len() {
                            result[index] = SupportLang::Kotlin;
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "lua", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < LUA_EXTS.len() {
                            result[index] = SupportLang::Lua;
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "php", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < PHP_EXTS.len() {
                            result[index] = SupportLang::Php;
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "python", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < PYTHON_EXTS.len() {
                            result[index] = SupportLang::Python;
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "ruby", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < RUBY_EXTS.len() {
                            result[index] = SupportLang::Ruby;
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "rust", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < RUST_EXTS.len() {
                            result[index] = SupportLang::Rust;
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "scala", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < SCALA_EXTS.len() {
                            result[index] = SupportLang::Scala;
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "swift", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < SWIFT_EXTS.len() {
                            result[index] = SupportLang::Swift;
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "typescript", feature = "all-parsers", feature = "typescript-napi", feature = "napi-compatible"))]
                    {
                        let mut i = 0;
                        while i < TYPESCRIPT_EXTS.len() {
                            result[index] = SupportLang::TypeScript;
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "tsx", feature = "all-parsers", feature = "tsx-napi", feature = "napi-compatible"))]
                    {
                        let mut i = 0;
                        while i < TSX_EXTS.len() {
                            result[index] = SupportLang::Tsx;
                            index += 1;
                            i += 1;
                        }
                    }

                    #[cfg(any(feature = "yaml", feature = "all-parsers"))]
                    {
                        let mut i = 0;
                        while i < YAML_EXTS.len() {
                            result[index] = SupportLang::Yaml;
                            index += 1;
                            i += 1;
                        }
                    }

                    let _ = index; // Mark index as used to avoid unused assignment warning
                    result
                }

                build_extension_to_lang_array()
            };
        }
);

// ========== Consts for Planned Features ==========
// these aren't yet implemented

/// List of files that DO NOT have an extension but are still associated with a language.
#[cfg(any(feature = "bash", feature = "all-parsers", feature = "ruby"))]
#[allow(unused_variables)]
const LANG_RELATIONSHIPS_WITH_NO_EXTENSION: &'static [(&'static str, SupportLang)] = &[
    #[cfg(any(feature = "bash", feature = "all-parsers"))]
    ("profile", SupportLang::Bash),
    #[cfg(any(feature = "bash", feature = "all-parsers"))]
    ("bash_login", SupportLang::Bash),
    #[cfg(any(feature = "bash", feature = "all-parsers"))]
    ("bash_logout", SupportLang::Bash),
    #[cfg(any(feature = "bash", feature = "all-parsers"))]
    ("bashrc", SupportLang::Bash),
    #[cfg(any(feature = "bash", feature = "all-parsers"))]
    ("profile", SupportLang::Bash),
    #[cfg(any(feature = "ruby", feature = "all-parsers"))]
    ("Rakefile", SupportLang::Ruby),
    #[cfg(any(feature = "ruby", feature = "all-parsers"))]
    ("Gemfile", SupportLang::Ruby),
    #[cfg(any(feature = "ruby", feature = "all-parsers"))]
    ("config.ru", SupportLang::Ruby),
];

/// Files whose presence can resolve language identification
#[cfg(any(all(feature = "cpp", feature = "c"), feature = "all-parsers"))]
#[allow(unused_variables)]
const LANG_FILE_INDICATORS: &'static [(&'static str, SupportLang)] = &[
    #[cfg(any(all(feature = "cpp", feature = "c"), feature = "all-parsers"))]
    ("conanfile.txt", SupportLang::Cpp),
    #[cfg(any(all(feature = "cpp", feature = "c"), feature = "all-parsers"))]
    ("vcpkg.json", SupportLang::Cpp),
    #[cfg(any(all(feature = "cpp", feature = "c"), feature = "all-parsers"))]
    ("CMakeLists.txt", SupportLang::Cpp),
    #[cfg(any(all(feature = "cpp", feature = "c"), feature = "all-parsers"))]
    (".vcxproj", SupportLang::Cpp),
];

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn no_duplicate_extensions() {
        let mut seen = std::collections::HashSet::new();
        for ext in EXTENSIONS {
            assert!(seen.insert(*ext), "Duplicate extension found: {}", ext);
        }
    }
}
