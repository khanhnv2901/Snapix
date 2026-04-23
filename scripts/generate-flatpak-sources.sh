#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_PATH="${ROOT_DIR}/flatpak/cargo-sources.json"
DOWNLOADED_GENERATOR="${ROOT_DIR}/.flatpak-builder-tools/flatpak-cargo-generator.py"

if command -v flatpak-cargo-generator >/dev/null 2>&1; then
  GENERATOR="flatpak-cargo-generator"
elif command -v flatpak-cargo-generator.py >/dev/null 2>&1; then
  GENERATOR="flatpak-cargo-generator.py"
else
  mkdir -p "$(dirname "${DOWNLOADED_GENERATOR}")"
  if [ ! -x "${DOWNLOADED_GENERATOR}" ]; then
    echo "Downloading flatpak-cargo-generator.py..." >&2
    curl -fsSL \
      https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py \
      -o "${DOWNLOADED_GENERATOR}"
    chmod +x "${DOWNLOADED_GENERATOR}"
  fi
  GENERATOR="${DOWNLOADED_GENERATOR}"
fi

cd "${ROOT_DIR}"
"${GENERATOR}" Cargo.lock -o "${OUTPUT_PATH}"
echo "Wrote ${OUTPUT_PATH}"
