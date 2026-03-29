#!/bin/bash
# Install ctx plugin into the Thread repo
# Run from wherever the extracted ctx-plugin/ directory is

set -euo pipefail

THREAD_DIR="${1:-$HOME/thread}"
PLUGIN_SRC="$(cd "$(dirname "$0")" && pwd)"

echo "Installing ctx plugin into: $THREAD_DIR"

# Create plugin directory structure
mkdir -p "$THREAD_DIR/plugins/ctx/.claude-plugin"
mkdir -p "$THREAD_DIR/plugins/ctx/commands"
mkdir -p "$THREAD_DIR/plugins/ctx/agents"
mkdir -p "$THREAD_DIR/plugins/ctx/skills"
mkdir -p "$THREAD_DIR/plugins/ctx/hooks"
mkdir -p "$THREAD_DIR/plugins/ctx/cross-client/.cursor/rules"
mkdir -p "$THREAD_DIR/plugins/ctx/cross-client/.codex"

# Copy all plugin files
cp "$PLUGIN_SRC/.claude-plugin/plugin.json"          "$THREAD_DIR/plugins/ctx/.claude-plugin/"
cp "$PLUGIN_SRC/commands/"*.md                        "$THREAD_DIR/plugins/ctx/commands/"
cp "$PLUGIN_SRC/agents/"*.md                          "$THREAD_DIR/plugins/ctx/agents/"
cp "$PLUGIN_SRC/skills/"*.md                          "$THREAD_DIR/plugins/ctx/skills/"
cp "$PLUGIN_SRC/hooks/"*.md                           "$THREAD_DIR/plugins/ctx/hooks/"
cp "$PLUGIN_SRC/cross-client/AGENTS.md"               "$THREAD_DIR/plugins/ctx/cross-client/"
cp "$PLUGIN_SRC/cross-client/.cursor/rules/ctx-audit.mdc" "$THREAD_DIR/plugins/ctx/cross-client/.cursor/rules/"
cp "$PLUGIN_SRC/cross-client/.codex/SKILL.md"         "$THREAD_DIR/plugins/ctx/cross-client/.codex/"
cp "$PLUGIN_SRC/README.md"                            "$THREAD_DIR/plugins/ctx/"
cp "$PLUGIN_SRC/PLAN.md"                              "$THREAD_DIR/plugins/ctx/"

echo ""
echo "Installed. Directory structure:"
find "$THREAD_DIR/plugins/ctx" -type f | sort | sed "s|$THREAD_DIR/||"
echo ""
echo "Next steps:"
echo "  1. Review the files in plugins/ctx/"
echo "  2. Test: cd $THREAD_DIR && claude '/ctx:discover'"
echo "  3. Iterate on the command prompts based on results"
