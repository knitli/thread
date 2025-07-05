#!/bin/bash
set -euo pipefail

declare ARG
ARG="${1:-pull}"

declare PREFIX TREE_MAIN_URL TREE_GRAMS_URL
declare -a LANGS GRAMMAR_REPOS REPO_LANGS
declare -A REPO BRANCH

PREFIX="--prefix=parsers/"
TREE_MAIN_URL="https://github.com/tree-sitter/tree-sitter"
TREE_GRAMS_URL="https://github.com/tree-sitter-grammars/tree-sitter"

# These are all master branches in the main tree-sitter repository

LANGS=("bash" "c" "cpp" "c-sharp" "css" "go" "haskell" "html" "java" "javascript" "json" "julia" "jsdoc" "lua" "python" "php" "ocaml" "regex" "ruby" "rust" "scala" "typescript")

# List of grammar repositories with their branches
GRAMMAR_REPOS+=("cuda,master")
GRAMMAR_REPOS+=("hcl,main")
GRAMMAR_REPOS+=("hlsl,master")
GRAMMAR_REPOS+=("kotlin,master")
GRAMMAR_REPOS+=("make,main")
GRAMMAR_REPOS+=("markdown,split_parser")
GRAMMAR_REPOS+=("meson,master")
GRAMMAR_REPOS+=("scss,master")
GRAMMAR_REPOS+=("svelte,master")
GRAMMAR_REPOS+=("toml,master")
GRAMMAR_REPOS+=("xml,master")
GRAMMAR_REPOS+=("vue,main")
GRAMMAR_REPOS+=("yaml,master")
GRAMMAR_REPOS+=("zig,master")

# List of languages with their repositories and branches
REPO_LANGS=("dockerfile" "elixir" "just" "nix" "nu" "pkl" "r" "solidity" "sql" "swift")

# indexed by language to github username, all are tree-sitter-lang
REPO["dockerfile"]="camdencheek"
BRANCH["dockerfile"]="main"
REPO["elixir"]="elixir-lang"
BRANCH["elixir"]="main"
REPO["just"]="IndianBoy42"
BRANCH["just"]="main"
REPO["nix"]="nix-community"
BRANCH["nix"]="master"
REPO["nu"]="nushell"
BRANCH["nu"]="main"
REPO["pkl"]="apple"
BRANCH["pkl"]="main"
REPO["r"]="r-lib"
BRANCH["r"]="main"
REPO["solidity"]="JoranHonig"
BRANCH["solidity"]="master"
REPO["sql"]="DerekStride"
BRANCH["sql"]="main"
REPO["swift"]="alex-pinkus"
BRANCH["swift"]="main"

error_exit() {
    echo "Error: $1"
    exit 1
}

# Get URL for languages in misc. repositories (REPO array)
get_repo() {
    local lang="$1"
    local repo="${REPO[$lang]}"
    if [[ -z "$repo" ]]; then
        error_exit "No repository found for language: $lang"
    fi
    echo "https://github.com/$repo/tree-sitter-$lang.git"
}

# Get URL for languages in main tree-sitter repository
get_main_repo() {
    local lang="$1"
    echo "$TREE_MAIN_URL-$lang.git" 2>/dev/null || {
        error_exit "Failed to get main repository URL for language: $lang"
    }
}

# Get URL for languages in tree-sitter-grammars repository
get_grammar_repo() {
    local lang="$1"
    echo "$TREE_GRAMS_URL-$lang.git" 2>/dev/null || {
        error_exit "Failed to get grammar repository URL for language: $lang"
    }
}

get_cmd() {
    local lang="$1"
    local url="$2"
    local action="$3"
    local branch="$4"
    echo "git subtree --squash $PREFIX/$lang $action $url $branch" 2>/dev/null || {
        error_exit "Failed to construct command for language: $lang"
    }
}

main() {
    echo "Running command: $ARG"

    for lang in "${LANGS[@]}"; do
        local repo_url cmd
        repo_url=$(get_main_repo "$lang")
        cmd=$(get_cmd "$lang" "$repo_url" "$ARG" "master")
        eval "$cmd" || {
            error_exit "Failed to process language: $lang"
        }
    done
    for lang in "${REPO_LANGS[@]}"; do
        local repo_url branch cmd
        repo_url=$(get_repo "$lang")
        branch=${BRANCH[$lang]:-main}
        cmd=$(get_cmd "$lang" "$repo_url" "$ARG" "$branch")
        eval "$cmd" || {
            error_exit "Failed to process language: $lang"
        }
    done
    for grammar in "${GRAMMAR_REPOS[@]}"; do
        IFS=',' read -r lang branch <<<"$grammar"
        repo_url=$(get_grammar_repo "$lang")
        cmd="$(get_cmd "$lang" "$repo_url" "$ARG" "$branch")"
        eval "$cmd" || {
            error_exit "Failed to process grammar: $grammar"
        }
    done
}

main
