#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_ID="io.github.snapix.Snapix"
MANIFEST_PATH="${ROOT_DIR}/flatpak/${APP_ID}.yml"
BUILD_DIR="${ROOT_DIR}/.flatpak-builder"
REPO_DIR="${ROOT_DIR}/.flatpak-repo"
DIST_DIR="${ROOT_DIR}/dist"
BUNDLE_PATH="${DIST_DIR}/${APP_ID}.flatpak"

for cmd in flatpak flatpak-builder; do
  if ! command -v "${cmd}" >/dev/null 2>&1; then
    echo "Missing required command: ${cmd}" >&2
    exit 1
  fi
done

"${ROOT_DIR}/scripts/generate-flatpak-sources.sh"

mkdir -p "${DIST_DIR}"

flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo

flatpak-builder \
  --user \
  --force-clean \
  --install-deps-from=flathub \
  --repo="${REPO_DIR}" \
  "${BUILD_DIR}" \
  "${MANIFEST_PATH}"

flatpak build-bundle "${REPO_DIR}" "${BUNDLE_PATH}" "${APP_ID}"
echo "Wrote ${BUNDLE_PATH}"
