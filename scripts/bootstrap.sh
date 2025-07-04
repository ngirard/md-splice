#!/bin/bash
# Prepares the agent's sandboxed environment for the aistudio2md project.
# 
# This script ensures the necessary task runners are present using the fastest
# available method (pre-compiled binaries), then delegates to the declarative
# Makefile.toml for the main setup.
# This script is idempotent and can be run multiple times safely.

set -euo pipefail

# --- Force execution from the project root directory ---
# This ensures that 'cargo make' and other commands find the correct files.
cd "$(dirname "$0")/.."

# --- Stage 1: Install the Fast Installer (cargo-binstall) ---
# We check for cargo-binstall. If it's not present, we install it using its
# own binary release installer, avoiding a slow compile from source.
if ! command -v cargo-binstall &> /dev/null; then
    echo "Fast installer 'cargo-binstall' not found. Installing..."
    curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
fi

# --- Stage 2: Install the Task Runner (cargo-make) ---
# Now, we use the fast installer to get cargo-make. This will download a
# pre-compiled binary instead of compiling it, which is much faster.
if ! command -v cargo-make &> /dev/null; then
    echo "Task runner 'cargo-make' not found. Installing via cargo-binstall..."
    # Add --no-confirm to prevent interactive prompts.
    cargo binstall --no-confirm cargo-make
fi

# --- Stage 3: Delegate to the Declarative Task Definition ---
# With our tooling in place, we run the main bootstrap task defined in Makefile.toml.
# The --no-workspace flag is crucial: it prevents cargo-make from
# iterating over workspace members, ensuring the task runs only once at the root.
echo "--- All tools are ready. Running main bootstrap task... ---"
cargo make --no-workspace bootstrap
