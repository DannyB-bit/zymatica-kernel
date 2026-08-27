#!/bin/bash
set -e

echo "============================================="
echo "       Zymatica Engine Installer             "
echo "============================================="

OS=$(uname -s)
ARCH=$(uname -m)

if [ "$OS" != "Linux" ]; then
    echo "Error: This installer currently supports Linux only."
    exit 1
fi

if [ "$ARCH" = "x86_64" ]; then
    TARGET="x86_64-unknown-linux-gnu"
elif [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
    TARGET="aarch64-unknown-linux-gnu"
else
    echo "Error: Unsupported architecture: $ARCH"
    exit 1
fi

echo "Detected target architecture: $TARGET"

# Query the GitHub API to fetch the latest release tag
LATEST_TAG=$(curl -s https://api.github.com/repos/DannyB-bit/Zymatica-Engine/releases/latest | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_TAG" ]; then
    LATEST_TAG="v0.2.0"
    echo "Warning: Could not fetch latest release tag from GitHub. Defaulting to: $LATEST_TAG"
else
    echo "Latest release found: $LATEST_TAG"
fi

URL="https://github.com/DannyB-bit/Zymatica-Engine/releases/download/${LATEST_TAG}/zymatica-engine-${TARGET}.tar.gz"

echo "Downloading release archive from: $URL"
curl -L -O "$URL"

echo "Extracting binary..."
tar -xzf "zymatica-engine-${TARGET}.tar.gz"

echo "Installing to /usr/local/bin/zymatica-engine (requires sudo permissions)..."
sudo mv zymatica-engine /usr/local/bin/
sudo chmod +x /usr/local/bin/zymatica-engine

echo "Cleaning up temporary downloads..."
rm "zymatica-engine-${TARGET}.tar.gz"

echo "============================================="
echo "   Installation Completed Successfully!      "
echo "============================================="
/usr/local/bin/zymatica-engine --version || true
