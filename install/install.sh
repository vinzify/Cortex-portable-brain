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

cortex_asset="cortex-app-${os}-${arch}"
sidecar_asset="rmvm-grpc-server-${os}-${arch}"

api="https://api.github.com/repos/${REPO}/releases"
if [[ "$VERSION" == "latest" ]]; then
  tag="$(curl -fsSL "$api/latest" | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n1)"
else
  tag="$VERSION"
fi

mkdir -p "$INSTALL_DIR"
base="https://github.com/${REPO}/releases/download/${tag}"
curl -fsSL "$base/${cortex_asset}" -o "$INSTALL_DIR/cortex"
curl -fsSL "$base/${cortex_asset}.sha256" -o /tmp/cortex.sha256
curl -fsSL "$base/${sidecar_asset}" -o "$INSTALL_DIR/rmvm-grpc-server"
curl -fsSL "$base/${sidecar_asset}.sha256" -o /tmp/rmvm.sha256
(
  cd "$INSTALL_DIR"
  expected="$(cut -d' ' -f1 /tmp/cortex.sha256)"
  actual="$(sha256sum cortex | awk '{print $1}')"
  [[ "$expected" == "$actual" ]]
  expected_rmvm="$(cut -d' ' -f1 /tmp/rmvm.sha256)"
  actual_rmvm="$(sha256sum rmvm-grpc-server | awk '{print $1}')"
  [[ "$expected_rmvm" == "$actual_rmvm" ]]
)
chmod +x "$INSTALL_DIR/cortex"
chmod +x "$INSTALL_DIR/rmvm-grpc-server"

echo "Installed cortex to $INSTALL_DIR/cortex"
echo "Installed rmvm-grpc-server to $INSTALL_DIR/rmvm-grpc-server"

if [[ -t 0 && -t 1 ]]; then
  echo "Running guided setup..."
  "$INSTALL_DIR/cortex" setup || true
fi
