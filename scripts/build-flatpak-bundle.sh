#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_ID="io.github.snapix.Snapix"
MANIFEST_PATH="${ROOT_DIR}/flatpak/${APP_ID}.yml"
DIST_DIR="${ROOT_DIR}/dist"
BUNDLE_PATH="${DIST_DIR}/${APP_ID}.flatpak"
WORK_DIR="$(mktemp -d /tmp/snapix-flatpak-build.XXXXXX)"
BUILD_DIR="${WORK_DIR}/build"
REPO_DIR="${WORK_DIR}/repo"
STATE_DIR="${WORK_DIR}/state"

cleanup() {
  rm -rf "${WORK_DIR}"
}

trap cleanup EXIT

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
  --state-dir="${STATE_DIR}" \
  --repo="${REPO_DIR}" \
  "${BUILD_DIR}" \
  "${MANIFEST_PATH}"

flatpak build-bundle "${REPO_DIR}" "${BUNDLE_PATH}" "${APP_ID}"
echo "Wrote ${BUNDLE_PATH}"
