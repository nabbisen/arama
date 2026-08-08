# Release Process

## Versioning

All workspace packages share a single version number (e.g. `0.36.2`). Members
inherit `[workspace.package].version`. Internal path dependencies use the broad
pre-1.0 requirement `version = "0"` needed for registry packaging; release
version bumps do not rewrite those dependency entries.

The version follows loose semantic versioning: minor bumps for new features,
patch bumps for fixes only. Project versions and Git tags use `X.Y.Z` without a
`v` prefix.

**The manifest version must equal the tag exactly.** The release workflow
fails the build otherwise — a `0.38.0` tag against a `0.37.0` manifest would
ship binaries that report the wrong version. Run `./version.sh --update
X.Y.Z` before tagging, every time.

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

### External FFmpeg artifact-absence gate

Arama releases must not bundle ffmpeg/ffprobe or direct the application to
download executable artifacts. Before publishing, inspect the application
archive and exercise setup on the available native platforms to confirm that
missing ffmpeg is reported as an external prerequisite and image-only use
remains available. Users retain responsibility for the license and provenance
of the external pair they select.

Run the reproducible source/package check before versioning. When evidence
archives or a built executable are available, pass each one explicitly:

```sh
bash scripts/check-external-ffmpeg-contract.sh
bash scripts/check-external-ffmpeg-contract.sh \
  --archive ../arama-X.Y.Z.tar.gz \
  --archive arama@Linux-x64-gnu-cpu-X.Y.Z.tar.gz \
  --binary target/release/arama
```

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

### 5. Package the archive (local sanity check)

CI produces the release's authoritative source archive (step 7 below).
Building it locally first is a pre-tag sanity check, not the release
action itself — it catches a layout or exclusion mistake before the tag
push commits to it.

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

### 7. Push the tag — this triggers the release

Commit steps 1–4 to `main`, then push a tag matching the release version,
with no `v` prefix:

```sh
git tag X.Y.Z
git push origin X.Y.Z
```

The tag push triggers `.github/workflows/release-executable.yaml`, which
**creates the release itself** (RFC 034) — nothing here is a manual `gh
release create` step. The workflow's required order:

```text
tag push
   │
   ▼
build all five executable variants + the source archive
   │  (all succeeded)
   ▼
create the release
   │
   ▼
attach all six assets           ← --clobber, safe to re-run
   │
   ▼
verify the expected asset count
```

**Nothing is published until every variant has succeeded.** A failed
variant leaves the tag with no release — visible in the Actions run, and
recoverable by pushing a fix and re-running — rather than a published
release with some assets missing.

Before pushing a real release tag, `workflow_dispatch` against `main`
proves all five executable variants build without creating or uploading
anything (Part C). This is cheap pre-tag verification; a variant failure
after the tag exists costs a whole tag-and-retry cycle instead.

### 8. Verify the published release

```sh
gh release view X.Y.Z
```

Confirm all **six** assets are attached: five executables
(`arama@<variant>-X.Y.Z.<ext>`) and one source archive
(`arama-X.Y.Z.tar.gz`, the same contract-compliant archive from steps 5–6
— **not** GitHub's auto-generated "Source code" links, which wrap in a
`<repo>-<sha>/` directory and do not satisfy the contract). The workflow's
own asset-count check makes this redundant in the ordinary case; run it
anyway as the human-facing confirmation.

## Distribution contracts

The three release channels have deliberately different contracts:

| Channel | Naming | Layout or resolution | Produced by |
|---|---|---|---|
| Source archive | `arama-X.Y.Z.tar.gz` | Project files at archive root; no wrapper | GitHub release executable workflow (`source-archive` job), from the tagged commit |
| Executable asset | `arama@<variant>-<tag>.<ext>` | One same-named wrapping directory containing the binary | GitHub release executable workflow (`build` job) |
| crates.io | `arama` plus internal `arama-*` packages | Published registry dependency graph | Owner-staged publication |

Both the source archive and the executable assets are produced and attached
by the same workflow, from the same tag push (RFC 034) — the owner's local
build in steps 5–6 is a pre-tag sanity check, not a separate release action.
GitHub's own auto-generated "Source code" archives are not the source-archive
channel; they always exist alongside a release regardless of this workflow
and carry a wrapping directory that does not satisfy this project's contract.

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
- [ ] Release artifacts contain no bundled ffmpeg/ffprobe executable, and
      setup exposes only the external prerequisite
- [ ] Workspace package version updated; internal dependency requirements unchanged
- [ ] `Cargo.lock` refreshed and reviewed without unrelated dependency churn
- [ ] Post-bump locked metadata/check gate passed
- [ ] `CHANGELOG.md` updated
- [ ] RFC files moved and status fields updated
- [ ] `rfcs/README.md` updated
- [ ] Local sanity archive built with files at the root (no parent directory)
- [ ] Local sanity archive excludes `.git/`, `.git-exclude/`, `target/`, and
      generated `docs/book/`
- [ ] `workflow_dispatch` run against `main` before tagging, for a release
      that changed platform-conditional code (Part C)
- [ ] Tag pushed (`X.Y.Z`, no `v` prefix); release-executable workflow
      completed with all jobs succeeding
- [ ] Published release has all six assets: five executables plus the
      contract-compliant source archive — verified via `gh release view`,
      not inferred from the workflow's own green checkmark
- [ ] Source, executable, and crates.io artifact contracts checked separately
- [ ] crates.io dry-run/publication follows dependency order with `arama` last
- [ ] `NOTICE` updated if new third-party components were added
