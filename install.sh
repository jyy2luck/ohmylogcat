#!/usr/bin/env sh
# Install ohmylogcat from the latest GitHub Release into ~/.local/bin (or $INSTALL_DIR).
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/jyy2luck/ohmylogcat/main/install.sh | sh
set -eu

REPO="jyy2luck/ohmylogcat"
INSTALL_DIR="${INSTALL_DIR:-${HOME}/.local/bin}"
API="https://api.github.com/repos/${REPO}/releases/latest"

os="$(uname -s)"
arch="$(uname -m)"

case "${os}" in
  Darwin)
    case "${arch}" in
      arm64|aarch64) asset="ohmylogcat-aarch64-apple-darwin.tar.gz" ;;
      x86_64) asset="ohmylogcat-x86_64-apple-darwin.tar.gz" ;;
      *)
        echo "Unsupported macOS architecture: ${arch}" >&2
        exit 1
        ;;
    esac
    ;;
  Linux)
    case "${arch}" in
      x86_64|amd64) asset="ohmylogcat-x86_64-unknown-linux-gnu.tar.gz" ;;
      aarch64|arm64) asset="ohmylogcat-aarch64-unknown-linux-gnu.tar.gz" ;;
      *)
        echo "Unsupported Linux architecture: ${arch}" >&2
        exit 1
        ;;
    esac
    echo "Note: Linux builds are not published yet. Build from source with: cargo install --git https://github.com/${REPO}" >&2
    exit 1
    ;;
  *)
    echo "Unsupported OS: ${os}" >&2
    echo "On Windows, download the .zip from: https://github.com/${REPO}/releases/latest" >&2
    exit 1
    ;;
esac

if ! command -v curl >/dev/null 2>&1; then
  echo "curl is required" >&2
  exit 1
fi

tmpdir="$(mktemp -d)"
trap 'rm -rf "${tmpdir}"' EXIT

echo "Fetching latest release metadata..."
json="$(curl -fsSL "${API}")"

# Prefer downloading via browser_download_url for the matching asset.
url="$(printf '%s' "${json}" | sed -n "s/.*\"browser_download_url\": \"\\([^\"]*${asset}\\)\".*/\\1/p" | head -n 1)"

if [ -z "${url}" ]; then
  tag="$(printf '%s' "${json}" | sed -n 's/.*"tag_name": "\([^"]*\)".*/\1/p' | head -n 1)"
  if [ -z "${tag}" ]; then
    echo "Could not find a GitHub release. Publish a tag like v0.1.0 first." >&2
    exit 1
  fi
  url="https://github.com/${REPO}/releases/download/${tag}/${asset}"
fi

echo "Downloading ${asset}..."
curl -fsSL "${url}" -o "${tmpdir}/${asset}"

echo "Extracting..."
tar -xzf "${tmpdir}/${asset}" -C "${tmpdir}"

if [ ! -f "${tmpdir}/ohmylogcat" ]; then
  echo "Archive did not contain ohmylogcat binary" >&2
  exit 1
fi

mkdir -p "${INSTALL_DIR}"
install -m 755 "${tmpdir}/ohmylogcat" "${INSTALL_DIR}/ohmylogcat"

echo "Installed to ${INSTALL_DIR}/ohmylogcat"

case ":${PATH}:" in
  *":${INSTALL_DIR}:"*) ;;
  *)
    echo "Add this to your shell profile if needed:"
    echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    ;;
esac

echo "Requires adb (Android SDK platform-tools) on PATH or configured in Settings."
echo "Run: ohmylogcat"
