#!/usr/bin/env bash
set -euo pipefail

REPO="${CORTEX_BRAIN_REPO:-vinzify/Cortex-portable-brain}"
VERSION="${1:-latest}"
INSTALL_DIR="${CORTEX_INSTALL_DIR:-$HOME/.local/bin}"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"
case "$OS" in
  linux*) os="linux" ;;
  darwin*) os="macos" ;;
  *) echo "Unsupported OS: $OS"; exit 1 ;;
esac
case "$ARCH" in
  x86_64|amd64) arch="x64" ;;
  arm64|aarch64) arch="arm64" ;;
  *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

asset="cortex-app-${os}-${arch}"
if [[ "$os" == "windows" ]]; then
  asset="${asset}.exe"
fi

api="https://api.github.com/repos/${REPO}/releases"
if [[ "$VERSION" == "latest" ]]; then
  tag="$(curl -fsSL "$api/latest" | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n1)"
else
  tag="$VERSION"
fi

mkdir -p "$INSTALL_DIR"
base="https://github.com/${REPO}/releases/download/${tag}"
curl -fsSL "$base/${asset}" -o "$INSTALL_DIR/cortex"
curl -fsSL "$base/${asset}.sha256" -o /tmp/cortex.sha256
(
  cd "$INSTALL_DIR"
  expected="$(cut -d' ' -f1 /tmp/cortex.sha256)"
  actual="$(sha256sum cortex | awk '{print $1}')"
  [[ "$expected" == "$actual" ]]
)
chmod +x "$INSTALL_DIR/cortex"

echo "Installed cortex to $INSTALL_DIR/cortex"
