#!/bin/sh
set -e

# project-dots (v0.1.0-beta.1) System Installer Script
# Usage: curl -sSL https://raw.githubusercontent.com/user/project-dots/main/install.sh | sh

REPO="${PROJECT_DOTS_REPO:-user/project-dots}"
INSTALL_DIR="$HOME/.local/bin"

echo "=== project-dots System Installer ==="

# 1. Platform & Architecture Detection
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux*)
        if [ -d "/data/data/com.termux/files/usr" ]; then
            PLATFORM="termux"
            TARGET_ASSET="project-dots-aarch64-linux-android.tar.gz"
        else
            PLATFORM="debian"
            if [ "$ARCH" = "x86_64" ]; then
                TARGET_ASSET="project-dots-x86_64-unknown-linux-gnu.tar.gz"
            else
                TARGET_ASSET="project-dots-aarch64-linux-android.tar.gz"
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

echo "\n\033[1;32mInstallation completed successfully!\033[0m"
echo "Binary installed to: $INSTALL_DIR/project-dots"

# 4. PATH Environment Check
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
        echo "\n\033[1;33m[TIP] '$INSTALL_DIR' is not currently in your $PATH environment variable!\033[0m"
        echo "To use 'project-dots' from anywhere, add it to your shell config:"
        echo "  - bash: echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc"
        echo "  - zsh:  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zshrc"
        echo "  - fish: fish_add_path \$HOME/.local/bin"
        ;;
esac
