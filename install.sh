#!/bin/sh
set -e

# project-dots (v0.1.0-beta.32) System Installer Script
# Usage: curl -fsSL https://raw.githubusercontent.com/fusoras/project-dots/develop/install.sh | sh

# ── Dependency check ──────────────────────────────────────────
check_dependencies() {
    missing=""
    for cmd in curl tar grep sed; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            missing="$missing $cmd"
        fi
    done
    if [ -n "$missing" ]; then
        printf 'Missing required tools:%s\n' "$missing" >&2
        exit 1
    fi
}
check_dependencies

REPO="${DOTSS_REPO:-${PROJECT_DOTS_REPO:-fusoras/project-dots}}"
INSTALL_DIR="$HOME/.local/bin"

echo "=== dotss System Installer ==="

# 1. Platform & Architecture Detection
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux*)
        if [ -d "/data/data/com.termux/files/usr" ]; then
            PLATFORM="termux"
            INSTALL_DIR="${PREFIX:-$HOME/.local}/bin"
            TARGET_ASSET="dotss-aarch64-unknown-linux-musl.tar.gz"
        else
            PLATFORM="debian"
            INSTALL_DIR="$HOME/.local/bin"
            if [ "$ARCH" = "x86_64" ]; then
                TARGET_ASSET="dotss-x86_64-unknown-linux-gnu.tar.gz"
            else
                TARGET_ASSET="dotss-aarch64-unknown-linux-musl.tar.gz"
            fi
        fi
        ;;
    *)
        echo "Error: Unsupported operating system '$OS'." >&2
        exit 1
        ;;
esac

echo "Detected Platform: $PLATFORM ($ARCH)"
echo "Target Binary Asset: $TARGET_ASSET"

# 2. Fetch Latest Release Version Tag
RELEASE_API="https://api.github.com/repos/$REPO/releases/latest"
echo "Querying latest release from $RELEASE_API..."

TAG_NAME=$(curl -fsSL -H "User-Agent: dotss-installer" "$RELEASE_API" 2>/dev/null | grep '"tag_name":' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/' || true)

if [ -z "$TAG_NAME" ]; then
    TAG_NAME="v0.1.0-beta.32"
fi
echo "Installing release version: $TAG_NAME"

DOWNLOAD_URL="https://github.com/$REPO/releases/download/$TAG_NAME/$TARGET_ASSET"

# 3. Create Installation Directory & Download Asset
mkdir -p "$INSTALL_DIR"
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT INT TERM

echo "Downloading binary payload..."
curl -fSL -o "$TMP_DIR/dotss.tar.gz" "$DOWNLOAD_URL" || {
    echo "Error: Failed to download release asset from $DOWNLOAD_URL" >&2
    exit 1
}

echo "Extracting binary to $INSTALL_DIR..."
tar -xzf "$TMP_DIR/dotss.tar.gz" -C "$TMP_DIR"

if [ ! -f "$TMP_DIR/dotss" ]; then
    echo "Error: extracted archive did not contain 'dotss' binary" >&2
    exit 1
fi

mv "$TMP_DIR/dotss" "$INSTALL_DIR/dotss"
chmod +x "$INSTALL_DIR/dotss"

echo ""
printf "\033[1;32mInstallation completed successfully!\033[0m\n"
echo "Binary installed to: $INSTALL_DIR/dotss"
echo ""
printf "\033[1;33m[TIP] Add this line to your ~/.zshrc or ~/.bashrc\033[0m\n"
echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
