#!/usr/bin/env bash
set -euo pipefail

REPO="${IMAGO_REPO:-parkjangwon/imago}"
VERSION="${IMAGO_VERSION:-latest}"
BINARY_NAME="imago"
DEFAULT_INSTALL_DIR="/usr/local/bin"
FALLBACK_INSTALL_DIR="${HOME}/.local/bin"
INSTALL_DIR="${IMAGO_INSTALL_DIR:-$DEFAULT_INSTALL_DIR}"

if [[ "${1:-}" == "--help" ]]; then
  cat <<USAGE
imago installer

Usage:
  curl -fsSL https://raw.githubusercontent.com/${REPO}/main/install.sh | bash

Env vars:
  IMAGO_REPO         GitHub repo (default: parkjangwon/imago)
  IMAGO_VERSION      Tag like v1.0.0 or latest (default: latest)
  IMAGO_INSTALL_DIR  Install dir (default: /usr/local/bin, auto-fallback to ~/.local/bin)
USAGE
  exit 0
fi

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Darwin)
    if [[ "$ARCH" == "arm64" ]]; then
      TARGET="aarch64-apple-darwin"
      EXT="tar.gz"
    else
      echo "Unsupported macOS arch: $ARCH (only Apple Silicon supported)" >&2
      exit 1
    fi
    ;;
  Linux)
    if [[ "$ARCH" == "x86_64" ]]; then
      TARGET="x86_64-unknown-linux-gnu"
      EXT="tar.gz"
    else
      echo "Unsupported Linux arch: $ARCH (only x86_64 supported)" >&2
      exit 1
    fi
    ;;
  MINGW*|MSYS*|CYGWIN*|Windows_NT)
    TARGET="x86_64-pc-windows-msvc"
    EXT="zip"
    ;;
  *)
    echo "Unsupported OS: $OS" >&2
    exit 1
    ;;
esac

if [[ "$VERSION" == "latest" ]]; then
  VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | sed -n 's/.*"tag_name": "\([^"]*\)".*/\1/p' | head -n1)"
fi

if [[ -z "$VERSION" ]]; then
  echo "Could not resolve release version." >&2
  exit 1
fi

ASSET="${BINARY_NAME}-${TARGET}.${EXT}"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"

TMP_DIR="$(mktemp -d)"
cleanup() { rm -rf "$TMP_DIR"; }
trap cleanup EXIT

echo "Installing ${BINARY_NAME} ${VERSION} (${TARGET})..."
curl -fL "$URL" -o "$TMP_DIR/$ASSET"

case "$EXT" in
  tar.gz)
    tar -xzf "$TMP_DIR/$ASSET" -C "$TMP_DIR"
    SRC="$TMP_DIR/$BINARY_NAME"
    ;;
  zip)
    unzip -q "$TMP_DIR/$ASSET" -d "$TMP_DIR"
    SRC="$TMP_DIR/${BINARY_NAME}.exe"
    ;;
esac

if [[ ! -f "$SRC" ]]; then
  echo "Binary not found in archive: $ASSET" >&2
  exit 1
fi

install_binary_unix() {
  local src="$1"
  local dst_dir="$2"
  mkdir -p "$dst_dir"
  install -m 755 "$src" "$dst_dir/$BINARY_NAME"
  echo "✅ Installed to $dst_dir/$BINARY_NAME"
  "$dst_dir/$BINARY_NAME" --version || true

  case ":$PATH:" in
    *":$dst_dir:"*) ;;
    *)
      echo "ℹ️  '$dst_dir' is not in PATH."
      echo "   Add this line to your shell profile (~/.zshrc or ~/.bashrc):"
      echo "   export PATH=\"$dst_dir:\$PATH\""
      ;;
  esac
}

if [[ "$OS" == "Darwin" || "$OS" == "Linux" ]]; then
  if [[ -n "${IMAGO_INSTALL_DIR:-}" ]]; then
    # User explicitly chose a dir: fail fast if it doesn't work.
    install_binary_unix "$SRC" "$INSTALL_DIR"
  else
    # Default behavior: try /usr/local/bin, fallback to ~/.local/bin on permission errors.
    if install_binary_unix "$SRC" "$INSTALL_DIR" 2>/dev/null; then
      :
    else
      echo "⚠️  No write permission to $INSTALL_DIR. Falling back to $FALLBACK_INSTALL_DIR"
      install_binary_unix "$SRC" "$FALLBACK_INSTALL_DIR"
    fi
  fi
else
  mkdir -p "$INSTALL_DIR"
  cp "$SRC" "$INSTALL_DIR/${BINARY_NAME}.exe"
  echo "✅ Installed to $INSTALL_DIR/${BINARY_NAME}.exe"
fi
