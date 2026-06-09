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

Use `version.sh` to update every member `Cargo.toml` atomically:

```sh
./version.sh --update X.Y.Z
```

Preview the change first with `--dry-run` if needed.

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

```sh
cd <workspace-root>
tar czf arama-vX.Y.Z.tar.gz \
  --exclude='arama-<src-dir>/target' \
  --transform='s|^arama-<src-dir>|arama-vX.Y.Z|' \
  arama-<src-dir>
```

The `--transform` flag renames the inner directory from the source
folder name to `arama-vX.Y.Z/` so the archive structure matches:

```
arama-vX.Y.Z.tar.gz
└── arama-vX.Y.Z/
    ├── Cargo.toml
    ├── app/
    ├── crates/
    └── ...
```

### 6. Verify the archive

```sh
tar tzf arama-vX.Y.Z.tar.gz | head -5
```

Confirm the first path is `arama-vX.Y.Z/`.

## Checklist

- [ ] All tests pass
- [ ] Version bumped in all `Cargo.toml` files
- [ ] `CHANGELOG.md` updated
- [ ] RFC files moved and status fields updated
- [ ] `rfcs/README.md` updated
- [ ] Archive created and inner path verified
- [ ] `NOTICE` updated if new third-party components were added
