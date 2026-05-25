#!/bin/bash
# One-liner: curl -fsSL https://aimf.io/install.sh | bash

set -e

VERSION=${VERSION:-latest}
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
  linux)
    case "$ARCH" in
      x86_64) FILE="aimf-linux-x86_64" ;;
      aarch64) FILE="aimf-linux-arm64" ;;
      *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
    esac
    ;;
  darwin)
    case "$ARCH" in
      x86_64) FILE="aimf-macos-x86_64" ;;
      arm64) FILE="aimf-macos-arm64" ;;
      *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
    esac
    ;;
  windows)
    FILE="aimf-windows-x86_64.exe"
    ;;
  *)
    echo "Unsupported OS: $OS"
    exit 1
    ;;
esac

# Download binary
URL="https://github.com/ai-mf/media-engine/releases/${VERSION}/download/${FILE}"
echo "⬇️  Downloading AIMF from $URL"
curl -fsSL "$URL" -o aimf
chmod +x aimf

# Move to PATH
if [ -w /usr/local/bin ]; then
  sudo mv aimf /usr/local/bin/
else
  mkdir -p ~/.local/bin
  mv aimf ~/.local/bin/
  echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
fi

echo "✅ AIMF installed successfully!"
aimf --version