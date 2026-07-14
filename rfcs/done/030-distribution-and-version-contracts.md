# RFC 030: Distribution and version contract reconciliation

**Status:** Implemented (Unreleased)

## Summary

Arama has three distinct distribution routes: a root-layout source archive,
wrapped platform executable assets, and a crates.io package graph. This RFC
documents those contracts consistently and keeps workspace version maintenance
independent of the number of internal crates.

All workspace packages inherit one `[workspace.package].version`. Internal
workspace dependencies retain `version = "0"` beside their local paths for
registry packaging. `version.sh` updates only the inherited workspace package
version; it never enumerates or rewrites internal dependencies.

## Context

Before this RFC, user and maintainer instructions blurred two different
archive layouts:

- the owner-created source archive stores project files at archive root;
- GitHub executable assets contain one same-named wrapping directory.

The executable workflow also produced a platform matrix that was not presented
clearly in the installation guide. crates.io installation was mentioned without
explaining that Cargo builds the published package graph locally.

Version-helper documentation was stale. Workspace members already inherit the
single package version, but docs claimed that the helper edited member
manifests, `Cargo.lock`, and the Git index. Earlier revisions of this RFC also
made the helper enumerate and synchronize every internal crate. The owner
rejected that topology coupling because each added crate would require helper,
fixture, and documentation maintenance.

## Decision

### 1. Keep the three distribution contracts distinct

| Channel | Naming | Layout or resolution | Produced by |
|---|---|---|---|
| Source archive | `arama-X.Y.Z.tar.gz` | Project files at archive root; no wrapper directory | Owner source-release step |
| Executable asset | `arama@<variant>-<tag>.<ext>` | One same-named wrapping directory containing the executable | GitHub executable workflow |
| crates.io | `arama` plus internal `arama-*` packages | Cargo resolves and builds the published graph | Owner-staged publication |

The user installation guide documents these as separate routes rather than
presenting one archive layout as universal. Project versions and Git tags use
`X.Y.Z` without a `v` prefix; source and executable asset names preserve that
tag text.

### 2. Document the executable asset matrix

The installation guide records the workflow's current assets:

- Linux x86_64 CPU;
- Linux x86_64 CUDA;
- Apple Silicon macOS;
- Windows x86_64 CPU;
- Windows x86_64 CUDA.

The guide must be updated with the workflow whenever its matrix, archive
extension, or naming convention changes.

### 3. Keep internal dependency requirements topology-independent

Internal workspace dependency entries retain the form:

```toml
arama-ai = { version = "0", path = "crates/ai" }
```

The path supports local workspace development, while the version requirement
allows Cargo to normalize the dependency for registry packaging. The broad
pre-1.0 requirement is an intentional project policy. It is not synchronized
with `[workspace.package].version`.

Adding or removing an internal crate requires the normal workspace manifest
changes, but never a `version.sh` name list, dependency count, or validation
fixture update.

### 4. Keep one package-version source of truth

All members inherit:

```toml
[workspace.package]
version = "X.Y.Z"
```

`version.sh` has only two responsibilities:

- show that workspace package version;
- update that single field atomically when the owner eventually requests a
  release bump.

The helper does not modify workspace dependencies, member manifests,
`Cargo.lock`, the changelog, or the Git index. It uses an adjacent temporary
file, preserves the manifest mode, checks replacement failures, and cleans up
on failure.

### 5. Publish the package graph in dependency order

Dry-runs and any separately authorized publication use the current dependency
order:

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

Publishing dependencies before dependents ensures the registry can resolve
the newest internal APIs used by the dependent source. Registry propagation
may require pauses between levels. `arama` is published last.

If workspace topology changes, this publication order must be recalculated in
the release guide. That release knowledge does not belong in `version.sh`.

### 6. Do not create a special 0.36.3 remediation requirement

The broad internal `version = "0"` requirements in already-published 0.36.2
manifests match the continuing project policy. This RFC therefore does not
require a 0.36.3 repair release, targeted yanks, or a block on later 0.x
publication.

Any actual version bump, publication, yank, archive, executable asset, tag, or
push remains a separate owner-authorized release action.

## Consequences

### Benefits

- Normal development and future workspace growth do not create version-helper
  maintenance.
- A release bump changes one source-of-truth package version.
- Source and executable archive layouts are no longer confused.
- Users can choose an executable, Cargo installation, or source build with
  accurate expectations.
- Registry staging remains explicit and dependency ordered.

### Accepted trade-off

Cargo interprets `version = "0"` as the full pre-1.0 range. Registry resolution
can therefore select different internal release versions. The owner accepts
that behavior to keep internal version requirements independent of workspace
release bumps and topology.

Publication review must compensate operationally by staging dependencies
before dependents and verifying normalized manifests and a fresh installation
before treating a release as complete.

## Non-goals

- No application behavior or UI change.
- No new release automation framework.
- No internal crate list or count in the version helper.
- No workspace dependency version synchronization.
- No 0.36.3 maintenance branch or remediation release.
- No version bump, package publication, yank, archive, executable build, tag,
  commit, or push.
- No resolution of the separate macOS ffmpeg supply-chain question.

## Acceptance criteria

- Root internal path dependencies use `version = "0"`.
- `version.sh` reads and updates only `[workspace.package].version`.
- Adding a workspace crate does not require changing `version.sh`.
- Helper dry-run edits nothing and reports only `Cargo.toml`.
- Successful helper replacement preserves the manifest mode.
- Failed final replacement returns nonzero, preserves the original manifest,
  and leaves no temporary file.
- README source extraction creates a destination for the root-layout archive.
- Installation documentation distinguishes all three routes and records the
  executable asset matrix.
- Version, tag, source-archive, executable-asset, and RFC status examples use
  `X.Y.Z` without a `v` prefix.
- Release documentation keeps dependency-ordered registry staging without
  requiring synchronized internal version requirements or 0.36.3 remediation.
- RFC index and roadmap record the implementation as unreleased.

## Review evidence

```sh
sh -n version.sh
./version.sh --list
./version.sh --update 0.36.2 --dry-run
cargo metadata --no-deps --locked --format-version 1
cargo check --workspace --all-targets
cargo test --workspace
mdbook build docs
git diff --check
```

Implementation review should also use isolated manifest copies to verify that
an update changes only `[workspace.package].version`, retains all internal
`version = "0"` entries, preserves mode `0644`, and fails cleanly when the
final rename is fault-injected.

Package dry-runs remain evidence-bound by registry state. In the initial RFC
030 implementation review, default verification passed through `arama-ai`.
`arama-ui-widgets` then encountered APIs absent from the latest published
`arama-ai`; packaging-only `--no-verify` dry-runs passed for it and the three
remaining dependents. That boundary does not claim current registry-backed
installability.

## Implementation notes

The implementation reconciles source, executable, and crates.io installation
documentation; corrects root-layout source extraction; documents the current
asset matrix and dependency publication order; and makes the version helper a
single-field, topology-independent operation.

An earlier reviewed revision synchronized nine internal requirements with the
workspace version and required a 0.36.3 registry remediation. Before commit,
the owner clarified that internal requirements must remain `version = "0"` and
that adding a crate must never require changing `version.sh`. This implemented
revision records that correction as the durable project decision.
