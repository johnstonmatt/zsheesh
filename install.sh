#!/bin/sh
# Install zsheesh — a zsh-aware code formatter
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/johnstonmatt/zsheesh/main/install.sh | sh
#
# Environment variables:
#   ZSHEESH_VERSION  — version to install (default: latest)
#   INSTALL_DIR      — where to put the binary (default: /usr/local/bin)

set -eu

REPO="johnstonmatt/zsheesh"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

detect_target() {
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Darwin) os="apple-darwin" ;;
    Linux)  os="unknown-linux-gnu" ;;
    *)
      echo "Unsupported OS: $os" >&2
      exit 1
      ;;
  esac

  case "$arch" in
    x86_64|amd64) arch="x86_64" ;;
    arm64|aarch64) arch="aarch64" ;;
    *)
      echo "Unsupported architecture: $arch" >&2
      exit 1
      ;;
  esac

  echo "${arch}-${os}"
}

get_latest_version() {
  curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' \
    | head -1 \
    | sed 's/.*"tag_name": *"//;s/".*//'
}

main() {
  target="$(detect_target)"
  version="${ZSHEESH_VERSION:-$(get_latest_version)}"

  if [ -z "$version" ]; then
    echo "Could not determine latest version. Set ZSHEESH_VERSION manually." >&2
    exit 1
  fi

  url="https://github.com/${REPO}/releases/download/${version}/zsheesh-${target}.tar.gz"
  echo "Downloading zsheesh ${version} for ${target}..."

  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' EXIT

  curl -fsSL "$url" -o "${tmpdir}/zsheesh.tar.gz"
  tar xzf "${tmpdir}/zsheesh.tar.gz" -C "$tmpdir"

  if [ -w "$INSTALL_DIR" ]; then
    mv "${tmpdir}/zsheesh" "${INSTALL_DIR}/zsheesh"
  else
    echo "Installing to ${INSTALL_DIR} (requires sudo)..."
    sudo mv "${tmpdir}/zsheesh" "${INSTALL_DIR}/zsheesh"
  fi

  chmod +x "${INSTALL_DIR}/zsheesh"
  echo "Installed zsheesh to ${INSTALL_DIR}/zsheesh"
}

main
