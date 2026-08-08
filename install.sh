#!/bin/sh
set -e

# project-dots (v0.1.0-beta.1) System Installer Script
# Usage: curl -sSL https://raw.githubusercontent.com/fusoras/project-dots/develop/install.sh | sh

REPO="${PROJECT_DOTS_REPO:-fusoras/project-dots}"
INSTALL_DIR="$HOME/.local/bin"

echo "=== project-dots System Installer ==="

# 1. Platform & Architecture Detection
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux*)
        if [ -d "/data/data/com.termux/files/usr" ]; then
            PLATFORM="termux"
            INSTALL_DIR="${PREFIX:-$HOME/.local}/bin"
            TARGET_ASSET="project-dots-aarch64-unknown-linux-musl.tar.gz"
        else
            PLATFORM="debian"
            INSTALL_DIR="$HOME/.local/bin"
            if [ "$ARCH" = "x86_64" ]; then
                TARGET_ASSET="project-dots-x86_64-unknown-linux-gnu.tar.gz"
            else
                TARGET_ASSET="project-dots-aarch64-unknown-linux-musl.tar.gz"
            fi
        fi
        ;;
    *)
        echo "Error: Unsupported operating system '$OS'."
        exit 1
        ;;
esac

echo "Detected Platform: $PLATFORM ($ARCH)"
echo "Target Binary Asset: $TARGET_ASSET"

# 2. Fetch Latest Release Version Tag
RELEASE_API="https://api.github.com/repos/$REPO/releases/latest"
echo "Querying latest release from $RELEASE_API..."

TAG_NAME=$(curl -sSL -H "User-Agent: project-dots-installer" "$RELEASE_API" | grep '"tag_name":' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/' || true)

if [ -z "$TAG_NAME" ]; then
    TAG_NAME="v0.1.0-beta.1"
fi
echo "Installing release version: $TAG_NAME"

DOWNLOAD_URL="https://github.com/$REPO/releases/download/$TAG_NAME/$TARGET_ASSET"

# 3. Create Installation Directory & Download Asset
mkdir -p "$INSTALL_DIR"
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

echo "Downloading binary payload..."
curl -sSL -o "$TMP_DIR/project-dots.tar.gz" "$DOWNLOAD_URL" || {
    echo "Error: Failed to download release asset from $DOWNLOAD_URL"
    exit 1
}

echo "Extracting binary to $INSTALL_DIR..."
tar -xzf "$TMP_DIR/project-dots.tar.gz" -C "$TMP_DIR"
mv "$TMP_DIR/project-dots" "$INSTALL_DIR/project-dots"
chmod +x "$INSTALL_DIR/project-dots"

echo ""
printf "\033[1;32mInstallation completed successfully!\033[0m\n"
echo "Binary installed to: $INSTALL_DIR/project-dots"
echo ""
printf "\033[1;33m[SHELL CONFIGURATION TIP]\033[0m\n"
echo "To ensure 'project-dots' is accessible from any terminal session, add it to your shell configuration:"
echo "  - bash: echo 'export PATH=\"$INSTALL_DIR:\$PATH\"' >> ~/.bashrc"
echo "  - zsh:  echo 'export PATH=\"$INSTALL_DIR:\$PATH\"' >> ~/.zshrc"
echo "  - fish: fish_add_path $INSTALL_DIR"
