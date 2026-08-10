#!/bin/sh
set -e

# ==============================================================================
# TEMPORARY TEST INSTALLER FOR BRANCH: feature/section-injections
# NOTE: Delete this file before merging feature/section-injections into develop/main.
# ==============================================================================

REPO="${DOTSS_REPO:-fusoras/project-dots}"
BRANCH="feature/section-injections"
INSTALL_DIR="$HOME/.local/bin"

echo "=== dotss Temporary Branch Installer (Branch: $BRANCH) ==="

OS="$(uname -s)"
ARCH="$(uname -m)"
echo "Detected Platform: $OS ($ARCH)"

mkdir -p "$INSTALL_DIR"

if command -v cargo >/dev/null 2>&1; then
    echo "Building and installing dotss directly from git branch '$BRANCH'..."
    cargo install --git "https://github.com/$REPO" --branch "$BRANCH" --force
    echo ""
    printf "\033[1;32mInstallation from branch '$BRANCH' completed successfully!\033[0m\n"
    echo "Binary installed to: $INSTALL_DIR/dotss"
    echo ""
    printf "\033[1;33m[TIP] Make sure \$HOME/.local/bin is in your \$PATH:\033[0m\n"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    exit 0
else
    echo "Error: 'cargo' is required to compile and install from branch '$BRANCH'." >&2
    exit 1
fi
