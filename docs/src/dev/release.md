# Release Process

## Versioning

All workspace members share a single version number (e.g. `0.24.0`).
The version follows loose semantic versioning: minor bumps for new
features, patch bumps for fixes only.

## Steps

### 1. Verify everything compiles and tests pass

```sh
cargo check --workspace
cargo test --workspace
```

### 2. Bump the version

The version is single-sourced in `[workspace.package].version`; every
member inherits it via `version.workspace = true`, so a bump is a
single one-line edit. Use the helper (no external tools required):

```sh
./version.sh --update X.Y.Z
```

Preview the change first with `--dry-run` if needed. Editing the
`[workspace.package].version` line in the root `Cargo.toml` by hand has
the same effect.

### 3. Update CHANGELOG.md

- Move the `[Unreleased]` items into a new `[X.Y.Z]` section.
- Open a fresh `[Unreleased]` section for the next cycle.
- Date is optional; the version number is sufficient.

### 4. Finalise RFC housekeeping

For any RFCs that ship in this release:
- Move `rfcs/proposed/NNN-slug.md` → `rfcs/done/`.
- Update the `**Status.**` field to `Implemented (vX.Y.Z)`.
- Add implementation notes if the as-built design deviated.
- Update `rfcs/README.md`.

### 5. Package the archive

From the workspace root, archive the project so the files sit at the
**root** of the tarball — no wrapping directory — so it unpacks
straight into the extraction destination:

```sh
cd <workspace-root>
tar --exclude='./target' -czf ../arama-vX.Y.Z.tar.gz .
```

The version number goes at the end of the archive name. The structure
must be:

```
arama-vX.Y.Z.tar.gz
├── Cargo.toml
├── app/
├── crates/
└── ...
```

### 6. Verify the archive

```sh
tar tzf arama-vX.Y.Z.tar.gz | head -5
```

Confirm the top-level entries are the project files themselves
(`./Cargo.toml`, `./app/`, …) and **not** a wrapping `arama-vX.Y.Z/`
directory.

## Checklist

- [ ] All tests pass
- [ ] Version bumped in `[workspace.package]` (members inherit it)
- [ ] `CHANGELOG.md` updated
- [ ] RFC files moved and status fields updated
- [ ] `rfcs/README.md` updated
- [ ] Archive created with files at the root (no parent directory)
- [ ] `NOTICE` updated if new third-party components were added
