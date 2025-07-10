#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
# SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
#
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Update licenses for files in the repository."""

import subprocess
from argparse import ArgumentParser, Namespace
from pathlib import Path
from datetime import datetime
from concurrent.futures import ThreadPoolExecutor


def parse_args() -> Namespace:
    """Parse command line arguments."""
    parser = ArgumentParser(
        description="Update licenses for files in the repository.")
    parser.add_argument(
        "--files",
        nargs="+",
        default=[],
        type=Path,
        help="List of files to update licenses for.",
    )
    return parser.parse_args()


ALLOWED_DIRS = [
    d
    for d in Path.cwd().iterdir()
    if d.is_dir()
    and (
        not d.name.startswith(".")
        or d.name in {".github", ".vscode", ".roo", ".claude"}
    )
]


def is_allowed_path(path: Path) -> bool:
    """Check if the path is allowed based on the allowed directories."""
    if (
        any(path.is_relative_to(allowed_dir) for allowed_dir in ALLOWED_DIRS)
        and path.is_file()
    ):
        return path in files if (files := parse_args().files) else True
    return False


def years() -> str:
    if (year := str(datetime.now().year)) and year != "2025":
        return f"2025-{year}"
    else:
        return "2025"


BASE_CMD = [
    "reuse",
    "annotate",
    "--year",
    str(datetime.now().year),
    "--copyright",
    "Knitli Inc. <knitli@knit.li>",
    "--fallback-dot-license",
    "--skip-existing",
]

AST_GREP_COPYRIGHT = (
    "Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>"
)

AST_GREP_PATHS = [
    path
    for path in (
        list((Path.cwd() / "crates" / "ast-grep").rglob("*.rs"))
        + list((Path.cwd() / "crates" / "languages").rglob("*.rs"))
    )
    if path.is_file() and is_allowed_path(path)
]

CODE_EXTS = {
    "rs",
    "sh",
    "py",
    "js",
    "ts",
    "tsx",
    "jsx",
    "java",
    "go",
    "c",
    "cpp",
    "h",
    "hpp",
    "html",
    "css",
    "svelte",
    "vue",
}

CODE_PATHS = [
    path
    for ext in CODE_EXTS
    for path in Path.cwd().rglob(f"*.{ext}")
    if path.is_file() and path not in AST_GREP_PATHS and is_allowed_path(path)
]
# Collect non-code paths that are not in the AST-Grep or code paths
# Some of these are shell scripts, so technically code, but we treat them as non-code for license purposes.
NON_CODE_EXTS = {
    "login",
    "astro",
    "bash",
    "bash_logout",
    "bashrc",
    "browserlistrc",
    "conf",
    "config",
    "csh",
    "css",
    "cts",
    "fish",
    "gitattributes",
    "gitmodules",
    "html",
    "htmx",
    "ini",
    "j2",
    "jinja",
    "jinja2",
    "json",
    "json5",
    "jsonc",
    "jsonl",
    "ksh",
    "md",
    "mdown",
    "mdtext",
    "mdtxt",
    "mdwn",
    "mdx",
    "mk",
    "mkd",
    "mts",
    "nix",
    "nu",
    "pkl",
    "profile",
    "quokka",
    "rs",
    "sass",
    "scss",
    "sh",
    "shellcheckrc",
    "sql",
    "sqlite",
    "stylelintrc",
    "tcsh",
    "toml",
    "txt",
    "yaml",
    "yml",
    "zlogin",
    "zlogout",
    "zprofile",
    "zsh",
    "zshenv",
    "zshrc",
}
NON_CODE_PATHS = [
    p
    for ext in NON_CODE_EXTS
    for p in Path.cwd().rglob(f"*.{ext}")
    if p.is_file() and p not in AST_GREP_PATHS and is_allowed_path(p)
]
print(
    f"Found {len(AST_GREP_PATHS)} AST-Grep paths, {len(CODE_PATHS)} code paths, and {
        len(NON_CODE_PATHS)
    } non-code paths."
)


def run_command(cmd: list[str], paths: list[Path]) -> None:
    """Run a command with the given paths."""
    if not paths:
        return
    cmds = [cmd + [str(path)] for path in paths]
    with ThreadPoolExecutor() as executor:
        executor.map(subprocess.run, cmds)


def main() -> None:
    """Main function to update licenses."""
    print("Updating licenses for code files...")
    if not AST_GREP_PATHS and not CODE_PATHS and not NON_CODE_PATHS:
        return
    if AST_GREP_PATHS:
        ast_grep_cmd = BASE_CMD + ["-c", AST_GREP_COPYRIGHT, "-l", "MIT"]
        run_command(ast_grep_cmd, AST_GREP_PATHS)
    if CODE_PATHS:
        code_cmd = BASE_CMD + ["-l", "AGPL-3.0-or-later"]
        run_command(code_cmd, CODE_PATHS)
    if NON_CODE_PATHS:
        non_code_cmd = BASE_CMD + ["-l", "MIT OR Apache-2.0"]
        run_command(non_code_cmd, NON_CODE_PATHS)
