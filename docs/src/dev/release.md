# Release Process

## Versioning

All workspace packages share a single version number (e.g. `0.36.2`). Members
inherit `[workspace.package].version`. Internal path dependencies use the broad
pre-1.0 requirement `version = "0"` needed for registry packaging; release
version bumps do not rewrite those dependency entries.

The version follows loose semantic versioning: minor bumps for new features,
patch bumps for fixes only. Project versions and Git tags use `X.Y.Z` without a
`v` prefix.

## Steps

### 1. Verify everything compiles and tests pass

```sh
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo audit
```

This is the default CPU release gate. `cargo clippy --workspace
--all-targets --all-features -- -D warnings` additionally enables the
`cuda` feature and requires a CUDA toolkit with `nvcc`; run it only in a
CUDA-equipped verification environment and record that separately.

If `cargo audit` passes with ignored advisories or allowed warnings,
confirm each `.cargo/audit.toml` ignore and each tracked warning still
has a current dependency path, rationale, and revisit condition.

For releases that include UI, setup, cache, first-run, or recoverable-error
changes, also consider the manual release smoke checklist in
[`testing.md`](./testing.md#release-smoke-with-the-ui). This is an
owner-managed confidence check, not a replacement for the automated gate and
not a release action by itself. Record an owner-run pass with the reusable
[release smoke evidence template](./release-smoke-evidence-template.md), using
`not run` or `environment-dependent` where the release environment cannot
exercise a check.

### Managed FFmpeg redistribution gate

The Linux/Windows managed artifacts currently selected in
`supported_artifacts()` are static `*-gpl` variants. Their upstream build
definition enables `--enable-gpl --enable-version3` and packages
`COPYING.GPLv3`; treat them as GNU GPL version 3 artifacts, not as an
LGPL/GPL-dependent choice.

Before publishing any release that redistributes these artifacts or directs
arama to download them, the release owner must review the exact pinned asset
identity and retain non-expiring evidence for its corresponding source. At a
minimum, confirm the distributed archive preserves its license and copyright
notices and provide recipients clear, equivalent access to the exact source,
dependency source, patches, and build scripts needed to reproduce that binary.
Do not rely only on a floating `master` or `latest` URL. Record the immutable
source/build identity and delivery location in the release review package.

The authoritative references are the upstream
[GPL build definition](https://github.com/yt-dlp/FFmpeg-Builds/blob/master/variants/defaults-gpl.sh),
[FFmpeg legal guidance](https://ffmpeg.org/legal.html), and
[GPLv3 terms](https://www.gnu.org/licenses/gpl-3.0.html). This project process
is a conservative release gate, not a legal opinion; unresolved corresponding
source or other license obligations block distribution.

### 2. Bump the version

Use the dependency-free helper to update `[workspace.package].version`:

```sh
./version.sh --list
./version.sh --update X.Y.Z --dry-run
./version.sh --update X.Y.Z
```

The helper does not edit workspace dependencies, `Cargo.lock`, member
manifests, the changelog, or the Git index. Adding or removing a workspace
crate does not require changing the helper. Refresh the committed lockfile
through Cargo, review the manifest and lockfile diff for only expected local
package-version changes, and then verify with the refreshed lock:

```sh
cargo check --workspace
git diff -- Cargo.toml Cargo.lock
cargo metadata --no-deps --locked
cargo check --workspace --locked
```

Stop if the lockfile contains unrelated dependency churn.

### 3. Update CHANGELOG.md

- Move the `[Unreleased]` items into a new `[X.Y.Z]` section.
- Open a fresh `[Unreleased]` section for the next cycle.
- Date is optional; the version number is sufficient.

### 4. Finalise RFC housekeeping

For any RFCs that ship in this release:
- Move `rfcs/proposed/NNN-slug.md` → `rfcs/done/`.
- Update the `**Status.**` field to `Implemented (X.Y.Z)`.
- Add implementation notes if the as-built design deviated.
- Update `rfcs/README.md`.

### 5. Package the archive

From the workspace root, archive the project so the files sit at the
**root** of the tarball — no wrapping directory — so it unpacks
straight into the extraction destination:

```sh
cd <workspace-root>
tar \
  --exclude='./target' \
  --exclude='./.git' \
  --exclude='./.git-exclude' \
  --exclude='./docs/book' \
  -czf ../arama-X.Y.Z.tar.gz .
```

The version number goes at the end of the archive name. The structure
must be:

```
arama-X.Y.Z.tar.gz
├── Cargo.toml
├── app/
├── crates/
└── ...
```

### 6. Verify the archive

```sh
tar tzf arama-X.Y.Z.tar.gz | head -5
```

Confirm the top-level entries are the project files themselves
(`./Cargo.toml`, `./app/`, …) and **not** a wrapping `arama-X.Y.Z/`
directory.

## Distribution contracts

The three release channels have deliberately different contracts:

| Channel | Naming | Layout or resolution | Produced by |
|---|---|---|---|
| Source archive | `arama-X.Y.Z.tar.gz` | Project files at archive root; no wrapper | Owner source-release step |
| Executable asset | `arama@<variant>-<tag>.<ext>` | One same-named wrapping directory containing the binary | GitHub release executable workflow |
| crates.io | `arama` plus internal `arama-*` packages | Published registry dependency graph | Owner-staged publication |

The executable workflow currently produces Linux x86_64 CPU/CUDA, Apple
Silicon macOS, and Windows x86_64 CPU/CUDA assets. Its wrapping directory is
not an exception to the source archive rule; it is a separate artifact type.

When the executable workflow matrix or naming changes, update the user
installation guide and this contract in the same reviewed change.

## crates.io lockstep procedure

Package dry-runs and any authorized publication follow this dependency order:

1. `arama-cache`
2. `arama-i18n`
3. `arama-env`
4. `arama-sidecar`
5. `arama-theme`
6. `arama-ai`
7. `arama-ui-widgets`
8. `arama-ui-layout`
9. `arama-ui-main`
10. `arama`

Every internal dependency must be available on crates.io before dry-running or
publishing its dependent at a new version. Registry propagation means a future
graph may need to proceed in stages: dry-run/publish the available dependency
level, wait until crates.io resolves it, then continue. Publish `arama` last.

Commands such as the following are evidence-only until the owner explicitly
authorizes publication:

```sh
cargo publish --dry-run -p arama-cache
cargo publish --dry-run -p arama-i18n
# Continue in the order above as registry prerequisites become available.
```

Before each actual publish, inspect the normalized package manifest and record
package, registry-availability, install, and publication evidence in the
release review package.

## Checklist

- [ ] All tests pass
- [ ] Manual UI smoke considered for UI/setup/cache/first-run changes; owner
      evidence recorded when run
- [ ] Managed FFmpeg GPLv3 artifact notices and exact corresponding-source
      delivery evidence reviewed for Linux/Windows distribution
- [ ] Workspace package version updated; internal dependency requirements unchanged
- [ ] `Cargo.lock` refreshed and reviewed without unrelated dependency churn
- [ ] Post-bump locked metadata/check gate passed
- [ ] `CHANGELOG.md` updated
- [ ] RFC files moved and status fields updated
- [ ] `rfcs/README.md` updated
- [ ] Archive created with files at the root (no parent directory)
- [ ] Archive excludes `.git/`, `.git-exclude/`, `target/`, and generated
      `docs/book/`
- [ ] Source, executable, and crates.io artifact contracts checked separately
- [ ] crates.io dry-run/publication follows dependency order with `arama` last
- [ ] `NOTICE` updated if new third-party components were added
