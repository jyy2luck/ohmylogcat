#!/usr/bin/env sh
# Install ohmylogcat from the latest GitHub Release into ~/.local/bin (or $INSTALL_DIR).
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/jyy2luck/ohmylogcat/main/install.sh | sh
# Optional local override for tests:
#   OHMYLOGCAT_INSTALL_SOURCE=/path/to/archive.tar.gz INSTALL_DIR=... sh install.sh
set -eu

REPO="jyy2luck/ohmylogcat"
INSTALL_DIR="${INSTALL_DIR:-${HOME}/.local/bin}"

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

archive_path="${tmpdir}/${asset}"
source_override="${OHMYLOGCAT_INSTALL_SOURCE:-}"

if [ -n "${source_override}" ]; then
  echo "Using local release archive ${source_override}..."
  if [ ! -f "${source_override}" ]; then
    echo "Local install source not found: ${source_override}" >&2
    exit 1
  fi
  cp "${source_override}" "${archive_path}"
else
  # Direct latest/download URL (avoids unauthenticated GitHub REST API rate limits).
  url="https://github.com/${REPO}/releases/latest/download/${asset}"
  echo "Downloading ${asset}..."
  echo "  from ${url}"
  if ! curl -fL --progress-bar "${url}" -o "${archive_path}"; then
    echo "Download failed for ${url}" >&2
    echo "Check network access and that a Release publishes asset: ${asset}" >&2
    exit 1
  fi
fi

echo "Extracting..."
tar -xzf "${archive_path}" -C "${tmpdir}"

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
