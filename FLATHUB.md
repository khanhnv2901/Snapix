# Flathub Submission

Snapix is ready to be submitted to Flathub using the files in `flatpak/`.

## Files used for submission

- `flatpak/io.github.snapix.Snapix.yml`
- `flatpak/cargo-sources.json`
- `flatpak/flathub.json`
- Upstream metadata already shipped in `data/`

## Before opening the PR

1. Regenerate Cargo sources after any Rust dependency change:
   `scripts/generate-flatpak-sources.sh`
2. Verify the Flatpak build locally:
   `scripts/build-flatpak-bundle.sh`
3. Commit `flatpak/cargo-sources.json` together with any manifest changes.

## Submitting to Flathub

1. Fork `flathub/flathub`.
2. Copy these files to the root of your forked submission branch:
   - `flatpak/io.github.snapix.Snapix.yml` -> `io.github.snapix.Snapix.yml`
   - `flatpak/cargo-sources.json` -> `cargo-sources.json`
   - `flatpak/flathub.json` -> `flathub.json`
3. Open a pull request for the new app submission.
4. On the PR, comment `bot, build` to trigger a test build after review feedback is addressed.

## Notes

- The submission is currently limited to `x86_64` in `flathub.json`.
- If `aarch64` is later validated, remove the architecture restriction.
