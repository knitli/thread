#!/usr/bin/env -S uv run --all-extras -s
# /// script
# requires-python = ">=3.11"
# dependencies = ["rignore", "cyclopts"]
# ///
# sourcery skip: avoid-global-variables
# SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
# SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
#
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Update licenses for files in the repository."""

import subprocess
import sys
from pathlib import Path
from functools import cache, partial
from datetime import datetime
from concurrent.futures import ThreadPoolExecutor
from typing import Annotated, NamedTuple

import rignore
from cyclopts import App, Parameter, validators

BASE_PATH = Path(__file__).parent.parent

__version__ = "0.1.0"

CONTRIBUTORS = Parameter(
    "-c",
    "--contributor",
    consume_multiple=True,
    help="Name and email of the contributor(s) to add. May be provided multiple times, or as a json list.",
    json_list=True,
)

app = App(
    name="Thread License Updater",
    version=__version__,
    default_command="add",
    help = "Update licenses for files in the repository using Reuse. Respects .gitignore.",
    help_on_error=True,
)

def run_command(cmd: list[str], paths: list[Path]) -> None:
    """Run a command with the given paths."""
    if not paths:
        return
    cmds = [cmd + [str(path)] for path in paths]
    with ThreadPoolExecutor() as executor:
        executor.map(subprocess.run, cmds)

def years() -> str:
    """
    Get the range of years for the copyright notice.
    """
    if (year := str(datetime.now().year)) and year != "2025":
        return f"2025-{year}"
    else:
        return "2025"

BASE_CMD = [
    "reuse",
    "annotate",
    "--year",
    years(),
    "--copyright",
    "Knitli Inc. <knitli@knit.li>",
    "--fallback-dot-license",
    "--merge-copyrights",
    "--skip-existing"
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

DEFAULT_CONTRIBUTORS = ["Adam Poulemanos <adam@knit.li>"]

AST_GREP_COPYRIGHT = (
    "Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>"
)


class PathsForProcessing(NamedTuple):
    """Paths for processing."""
    ast_grep_paths: list[Path]
    code_paths: list[Path]
    non_code_paths: list[Path]

    @classmethod
    def from_paths(cls, paths: tuple[list[Path], list[Path], list[Path]]) -> "PathsForProcessing":
        """Create an instance from a tuple of paths."""
        if len(paths) != 3:
            raise ValueError("Expected a tuple of three lists: (ast_grep_paths, code_paths, non_code_paths)")
        return cls(
            ast_grep_paths=paths[0],
            code_paths=paths[1],
            non_code_paths=paths[2],
        )

    def process_with_cmd(self, cmd: list[str]) -> None:
        """Run a command with the paths."""
        if not self.ast_grep_paths and not self.code_paths and not self.non_code_paths:
            return
        cmds = []
        if self.ast_grep_paths:
            ast_grep_cmd = cmd + ["-c", AST_GREP_COPYRIGHT, "-l", "AGPL-3.0-or-later AND MIT"]
            cmds.append((ast_grep_cmd, self.ast_grep_paths))
        if self.code_paths:
            code_cmd = cmd + ["-l", "AGPL-3.0-or-later"]
            cmds.append((code_cmd, self.code_paths))
        if self.non_code_paths:
            non_code_cmd = cmd + ["-l", "MIT OR Apache-2.0"]
            cmds.append((non_code_cmd, self.non_code_paths))
        for cmd, paths in cmds:
            run_command(cmd, paths)

AST_GREP_CRATES = ["crates/ast-engine", "crates/language", "crates/rule-engine"]

def get_staged_files() -> list[Path]:
    """Get the list of staged files in the git repository."""
    try:
        result = subprocess.run(
            ["git", "diff", "--cached", "--name-only"],
            capture_output=True,
            text=True,
            check=True
        )
        print(result.stdout.strip())
        staged_files = result.stdout.strip().splitlines()

        return [(BASE_PATH / file) for file in staged_files]
    except subprocess.CalledProcessError as e:
        print(f"Error getting staged files: {e}")
        return []

@cache
def filter_path(paths: tuple[Path] | None = None, path: Path | None = None) -> bool:
    """Check if a path is in the provided list of paths."""
    if not path:
        return False
    if paths is None:
        return path.is_file() and not path.is_symlink()
    return path in paths and path.is_file() and not path.is_symlink()

def get_empty_lists() -> tuple[list, list, list]:
    """Get empty lists for AST-Grep paths, code paths, and non-code paths."""
    return [], [], []

def sort_paths(paths: list[Path] | None = None, base_dir: Path = BASE_PATH) -> PathsForProcessing:
    """Sort paths by their string representation."""
    base_dir = base_dir or Path.cwd()
    ast_grep_paths, code_paths, non_code_paths = get_empty_lists()
    entry_filter = partial(filter_path, tuple(paths) if paths else None)
    for p in rignore.walk(base_dir, ignore_hidden = False, read_git_ignore=True, read_ignore_files=True, same_file_system=True):
        path = Path(p)
        if not entry_filter(path):
            continue
        if any(
            p
            for p in AST_GREP_CRATES
            if p in str(path) and p.suffix not in NON_CODE_EXTS
        ):
            ast_grep_paths.append(path)
        elif path.suffix in NON_CODE_EXTS:
            non_code_paths.append(path)
        else:
            code_paths.append(path)
    return PathsForProcessing.from_paths((ast_grep_paths, code_paths, non_code_paths))

def process_contributors(contributors: list[str]) -> list[str]:
    """Process contributors to ensure they are in the correct format."""
    processed = (item for contributor in contributors for item in ["--contributor", contributor])
    return list(processed)

@app.command(help="Update all licenses in the repository. Will check every file in the repository and add license information if it's missing.")
def update_all(*, contributors: Annotated[list[str], CONTRIBUTORS] = DEFAULT_CONTRIBUTORS) -> None:
    """Update all licenses in the repository."""
    path_obj = sort_paths()
    BASE_CMD.extend(process_contributors(contributors))
    try:
        path_obj.process_with_cmd(BASE_CMD)
    except Exception as e:
        print(f"Error updating licenses: {e}")

@app.command(help="Update licenses for staged files in the repository. Will only check files that are staged for commit.")
def staged(*, contributors: Annotated[list[str], CONTRIBUTORS] = DEFAULT_CONTRIBUTORS) -> None:
    """Update licenses for staged files in the repository."""
    staged_files = get_staged_files()
    if not staged_files:
        print("No staged files found.")
        sys.exit(0)
    path_obj = sort_paths(staged_files)
    BASE_CMD.extend(process_contributors(contributors))
    try:
        path_obj.process_with_cmd(BASE_CMD)
    except Exception as e:
        print(f"Error updating licenses: {e}")

@app.command(help="Add licenses for specific files in the repository. Will only check the files provided. May be provided as a space separated list, or as a json list. If a file already has a license, it will be skipped.")
def add(files: Annotated[list[Path], Parameter(validator=validators.Path(exists=True), required=True, consume_multiple=True, json_list=True)], *, contributors: Annotated[list[str], CONTRIBUTORS] = DEFAULT_CONTRIBUTORS) -> None:
    """Update licenses for specific files in the repository."""
    if not files:
        print("No files provided.")
        sys.exit(0)
    path_obj = sort_paths(files)
    BASE_CMD.extend(process_contributors(contributors))
    try:
        path_obj.process_with_cmd(BASE_CMD)
    except Exception as e:
        print(f"Error updating licenses: {e}")

def main() -> None:
    """Main function to update licenses."""
    app()

if __name__ == "__main__":
    main()
