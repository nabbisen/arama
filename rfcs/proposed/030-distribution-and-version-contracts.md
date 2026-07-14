# RFC 030 - Distribution and version contract reconciliation

**Status.** Proposed
**Tracks.** Documentation and release-operations reconciliation identified by
the 2026-07-14 architecture preparation review.
**Touches.** `README.md`, `docs/src/users/installation.md`,
`docs/src/dev/release.md`, `docs/src/dev/workflow.md`, `version.sh`,
`ROADMAP.md`, `rfcs/README.md`.

## Summary

arama currently describes three related contracts inconsistently:

1. Source release archives place project files at the archive root, but user
   instructions expect extraction to create an `arama-vX.Y.Z/` directory.
2. User documentation says there are no pre-built binaries, while the release
   workflow builds and uploads platform executable archives with a wrapping
   directory.
3. The workspace version is inherited from `[workspace.package]`, and internal
   path dependencies now use `version = "0"`, but `version.sh` and the workflow
   guide still describe exact internal-version rewrites and automatic staging.

This RFC defines the source archive, executable asset, and version-helper
contracts separately, then reconciles the affected documentation and helper
behavior without performing a release.

## Why

The current instructions are individually plausible but cannot all be true at
once. A user following the source quick start extracts files into the current
directory and then attempts to enter a directory that the compliant archive
does not contain. A user who wants a pre-built executable is told none exist,
although the release workflow creates them. A maintainer reading `version.sh`
cannot tell which manifest fields a version bump actually changes.

These are operational defects rather than product-code defects, but they affect
installation support and release safety. The contracts should be explicit
before another release is prepared.

## Current facts

### Source archives

Project policy and `docs/src/dev/release.md` require a source tarball named
`arama-vX.Y.Z.tar.gz` whose project files are at archive root, with no wrapping
directory.

`README.md` and `docs/src/users/installation.md` currently run:

```sh
tar xzf arama-vX.Y.Z.tar.gz
cd arama-vX.Y.Z
```

The second command fails for a compliant source archive unless the user first
creates and selects an extraction directory.

### Executable assets

`.github/workflows/release-executable.yaml` runs when a GitHub release is
created. It currently builds these asset variants:

| Platform | Architecture | Variant |
|---|---|---|
| Linux | x86_64 | CPU and CUDA |
| macOS | aarch64 | CPU |
| Windows | x86_64 | CPU and CUDA |

The asset basename is `arama@<matrix-name>-<tag>`. Each `.tar.gz` or `.zip`
contains a wrapping directory with that basename and the executable inside.
This differs intentionally from the source archive layout.

### Version metadata

All ten workspace packages inherit the release version from:

```toml
[workspace.package]
version = "0.36.2"
```

The nine internal dependency entries use local `path` resolution and carry a
broad 0.x version requirement as package metadata:

```toml
arama-ai = { version = "0", path = "crates/ai" }
```

The project does not publish the member crates as independently supported
libraries. The local path is authoritative inside this workspace. Exact
internal-version rewriting therefore adds release churn without improving the
current local build contract.

`version.sh` still attempts to replace an internal dependency only when its
version text equals the old workspace version. That condition cannot match the
current `version = "0"` entries. Its comments and help are therefore stale even
though the inherited workspace-package bump still works.

## Proposal

### Part A - Preserve the source archive root-layout contract

Keep the existing project rule: a source archive contains project files at its
root and has no wrapping directory.

Update source-build instructions to create an explicit destination directory:

```sh
mkdir arama-vX.Y.Z
tar xzf arama-vX.Y.Z.tar.gz -C arama-vX.Y.Z
cd arama-vX.Y.Z
```

Apply the same shape in `README.md` and `docs/src/users/installation.md`. Do not
change the source packaging command or introduce a parent directory into the
source tarball.

### Part B - Document two distribution channels

Document source and executable artifacts as separate deliverables:

| Contract | Source archive | Executable asset |
|---|---|---|
| Purpose | Build from source with Cargo | Run an owner-built platform binary |
| Naming | `arama-vX.Y.Z.tar.gz` | `arama@<platform-variant>-<tag>.<ext>` |
| Layout | Project files at archive root | One wrapping asset directory |
| Creation | Owner release process | GitHub release executable workflow |
| Coverage | Any supported source-build platform | Only the workflow matrix |

`docs/src/users/installation.md` should offer both routes and stop saying that
pre-built binaries do not exist. It should explain that executable assets cover
only the listed matrix and that CUDA variants require a compatible NVIDIA/CUDA
environment. The source route remains the portable fallback and contributor
route.

`README.md` should keep Quick Start concise. It may lead with the source route
and link to the installation guide for executable assets rather than duplicate
the full matrix.

`docs/src/dev/release.md` should explicitly separate:

- the owner-created source archive governed by the no-parent rule; and
- workflow-created executable assets governed by the wrapped-directory rule.

The source archive rule must not be generalized to executable assets, and the
workflow's wrapping directory must not be presented as a source-package
exception.

### Part C - Adopt the current internal version requirement deliberately

Retain `version = "0"` for internal workspace path dependencies. In this
project it means:

- package metadata carries a valid 0.x requirement alongside `path`;
- local workspace resolution remains authoritative;
- the requirement is not a promise that member crates are independently
  published or supported;
- an intentional future decision to publish member crates must revisit the
  requirement and publishing sequence.

Do not restore per-release exact versions in internal dependency entries under
this RFC.

### Part D - Simplify and document `version.sh`

Make `version.sh` match the single-source version model:

- `--list` reads `[workspace.package].version`;
- `--update X.Y.Z` rewrites only that field in the root `Cargo.toml`;
- `--dry-run` reports that single planned modification;
- the script does not rewrite internal `version = "0"` requirements;
- the script does not modify `Cargo.lock` directly;
- the script does not stage files with `git add`.

Update script comments/help and `docs/src/dev/workflow.md` accordingly. The
release process must refresh the committed `Cargo.lock` after a real version
bump, review that its local package-version changes are expected, and then
verify the workspace with the refreshed lock. That remains an explicit owner
step rather than hidden helper behavior.

Implementation should keep the helper jq-free and POSIX-shell compatible.

## Alternatives considered

### Wrap the source archive

Rejected because it contradicts the project-wide release archive contract and
would make existing release verification guidance wrong. Explicit destination
extraction fixes user ergonomics without changing the artifact.

### Flatten executable assets

Rejected for this reconciliation. The workflow intentionally packages each
binary in a named directory, and changing that automation would expand the
scope from documentation/contract repair into release-pipeline behavior.

### Restore exact internal dependency versions

Rejected for the current internal-only workspace. Exact versions previously
required nine additional textual updates on every release. The broad 0.x
requirement is sufficient while local paths are authoritative and member crates
are not independent deliverables.

### Retire `version.sh`

Not selected. The helper still provides a useful, dependency-free, consistent
way to read and update the single workspace version. Its scope only needs to be
made truthful.

## Non-goals

- No source or executable release creation.
- No version bump, changelog finalization, archive, tag, publish, or push.
- No change to `.github/workflows/release-executable.yaml` behavior or matrix.
- No promise of executable assets for platforms outside the current matrix.
- No publication of internal member crates.
- No macOS ffmpeg provider, checksum, license, or `NOTICE` decision; that
  supply-chain issue requires a separate security-focused RFC.
- No ELOC split or Rust source change.

## Risks

- Users may confuse source and executable layouts. Mitigation: use a compact
  contract table and label commands by artifact type.
- The executable workflow may drift later. Mitigation: cite its current matrix
  in developer docs and require release owners to reconcile documentation when
  the matrix changes.
- A broad internal 0.x requirement could be unsuitable if member crates become
  public deliverables. Mitigation: state that publication requires a new design
  decision and exact release sequencing.
- Simplifying `version.sh` could leave stale code paths or misleading help.
  Mitigation: validate list, dry-run, and real-update behavior against a
  temporary copy of `Cargo.toml`, and confirm the repository manifest remains
  unchanged during the test.

## Acceptance criteria

- Source extraction instructions work with a no-parent source archive by
  creating an explicit destination directory.
- User docs distinguish source builds from current workflow-produced executable
  assets, including naming, layout, matrix limits, and CUDA prerequisites.
- Developer release docs state separate source and executable artifact
  contracts without weakening the source archive rule.
- `version = "0"` remains the documented internal path-dependency requirement.
- `version.sh` reads and updates only `[workspace.package].version` and does not
  claim to update, stage, or directly lock any other file.
- `docs/src/dev/workflow.md` matches the helper's behavior.
- `docs/src/dev/release.md` explicitly refreshes and reviews `Cargo.lock` after
  a real version bump before the remaining release steps.
- No release mechanics or product behavior changes are included.
- Documentation builds cleanly.

## Review evidence

Required for proposal review:

```sh
mdbook build docs
git diff --check
```

Required for implementation review:

```sh
sh -n version.sh
./version.sh --list
./version.sh --update 0.36.2 --dry-run
mdbook build docs
git diff --check
```

Implementation review should additionally exercise `--update` against a
temporary copy of `Cargo.toml` and confirm that only
`[workspace.package].version` changes in that copy.
