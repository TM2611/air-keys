# Versioning and release strategy

## Scheme

Air Keys uses **Semantic Versioning** (SemVer): `MAJOR.MINOR.PATCH` (e.g. `1.0.0`, `1.1.0`, `2.0.0`).

- **MAJOR:** Incompatible API or behaviour changes.
- **MINOR:** New features, backwards compatible.
- **PATCH:** Bug fixes, backwards compatible.

## Where the version is defined

**Single source of truth:** Set the app version only in `package.json` (`version`).

A sync script copies it to `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml` whenever you run `npm run build` (or `npm run sync-version`). The release workflow runs the build, so CI stays in sync.

- **To bump the version:** Edit `package.json` only, then run `npm run build` or `npm run sync-version` before committing so the other two files are updated.
- **Files that must stay in sync for installers:** `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` (all updated by the sync script when you build).

## Branches

- **`main`** — Primary development branch.
- **`release/vX.Y`** — Release branch for version X.Y (e.g. `release/v1.0`, `release/v1.1`). Pushing to a branch matching `release/v*` triggers the Release workflow and produces installers. The workflow uses the version from the three files above.

## Tags

Tags like `v1.0.0` also trigger the Release workflow. Create the tag from the release branch when ready (e.g. from `release/v1.0` create tag `v1.0.0`).

## Release flow

1. Set the version in `package.json`. Run `npm run build` or `npm run sync-version` so `tauri.conf.json` and `Cargo.toml` are updated.
2. Merge or push to `release/vX.Y` (e.g. `release/v1.0`), or push a tag `vX.Y.Z`.
3. The Release workflow runs and creates a draft GitHub release with installers.
4. In GitHub, open the draft release, review, and publish when ready.
5. Users download installers from the [Releases](https://github.com/TM2611/air-keys/releases) page.
