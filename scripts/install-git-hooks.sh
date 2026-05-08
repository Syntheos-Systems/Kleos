#!/usr/bin/env bash
# Install the repo's tracked git hooks by pointing core.hooksPath at .githooks.
# Run once per clone.

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

git config core.hooksPath .githooks
chmod +x .githooks/*

echo "git hooks installed (core.hooksPath -> .githooks)"
echo "Active hooks:"
ls -1 .githooks
