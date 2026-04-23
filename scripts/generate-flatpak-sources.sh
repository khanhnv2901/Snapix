#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_PATH="${ROOT_DIR}/flatpak/cargo-sources.json"

if command -v flatpak-cargo-generator >/dev/null 2>&1; then
  GENERATOR="flatpak-cargo-generator"
elif command -v flatpak-cargo-generator.py >/dev/null 2>&1; then
  GENERATOR="flatpak-cargo-generator.py"
else
  echo "Missing flatpak cargo source generator." >&2
  echo "Install flatpak-builder-tools and expose flatpak-cargo-generator (or flatpak-cargo-generator.py) in PATH." >&2
  exit 1
fi

cd "${ROOT_DIR}"
"${GENERATOR}" Cargo.lock -o "${OUTPUT_PATH}"
echo "Wrote ${OUTPUT_PATH}"
