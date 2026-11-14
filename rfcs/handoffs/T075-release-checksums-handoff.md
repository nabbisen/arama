> **AUTHORIZED 2026-11-14 — owner:** *"It costs little with CI automation.
> Right? If so, approved."* (It does: one step in the existing `release` job.)
> Written 2026-11-14 by the architect.
>
> **Independent of every open task.** Proven end to end only by the next real
> release, which the owner and the architect cut — not by the dev team.

# T075 Handoff — a `SHA256SUMS` file on every release, checked against GitHub's own digests

## 1. Purpose

Give every release a `SHA256SUMS` file a user can check offline
(`sha256sum -c SHA256SUMS`), and have the release job **prove** that the files
GitHub stored are the files CI built, before the release is published.

## 2. Background

- **GitHub already records a SHA-256 for each asset** when it is uploaded, shown
  on the release page and in the API's `digest` field. Every 0.42.0 asset has
  one:
  `gh api repos/nabbisen/arama/releases/tags/0.42.0 --jq '.assets[] | "\(.name)\t\(.digest)"'`.
- **What is missing:** a checksum file that travels with a download, in the form
  every Linux and macOS user's `sha256sum`/`shasum` reads; and **any check that
  the uploaded bytes are the built bytes.** Today the job counts the assets
  (RFC 034 Part D) and never looks at their content.
- Found in review 165 (`.git-exclude/reviewed/165-t073-gui-session-and-fixtures-review.md`,
  P2 and its correction).

## 3. Applicable RFC and requirements

- [RFC 034](../done/034-release-workflow-reliability.md) — tag-triggered,
  idempotent re-runs (`--clobber`), asset-count verification (Part D).
- [RFC 037](../done/037-release-publication-atomicity.md) — the release is a
  draft until every claim is verified, and publication is the job's last step.
  **The new check is one of those claims: it runs before publishing.**

## 4. Change scope

- `.github/workflows/release-executable.yaml` — the `release` job and
  `EXPECTED_ASSET_COUNT`.
- `.github/workflows/scripts/release-checksums.sh` (new) — the two operations
  (§6), as a script so they can be tested without a tag.
- `docs/src/users/installation.md` — *Route 1*: how to verify a download.
- `docs/src/dev/release.md` — the checklist item.

## 5. Explicit non-change scope

- The `build`, `package-msix` and `source-archive` jobs, and the asset names.
- **Signing.** A checksum file served from the same release protects against a
  corrupted download, not against a compromised release. Signing is a separate
  decision, not this task's.
- `packaging/linux/arama-bin/PKGBUILD` and `packaging/linux/PKGBUILD`. Their
  first `SKIP` is deliberate (the comment there explains why), and the AUR's own
  copy gets the real hash at publish time.
- Cutting or re-running a release.

## 6. Required implementation

`release-checksums.sh` with two subcommands:

1. **`write <dir>`** — writes `<dir>/SHA256SUMS` over every regular file in
   `<dir>` except itself: sorted by name, in `sha256sum`'s own output format
   (`<hex>  <name>`, names with no directory part). Then it runs `sha256sum -c`
   on the result and fails if any line fails. Fails on an empty directory.
2. **`verify <dir> <assets.json>`** — `<assets.json>` is the release's asset list
   (`gh release view <tag> --json assets`). For **every** file in `<dir>`,
   `SHA256SUMS` included, the asset of that name must exist and its `digest` must
   equal `sha256:<local hex>`. Fails on: a missing asset, a `null` or absent
   `digest` (never treated as a pass), a mismatch, or an asset on the release that
   is not in `<dir>`. Each failure names the file and both values.

In the `release` job:

- After *Download all release assets*: `release-checksums.sh write dist`.
- `EXPECTED_ASSET_COUNT` 7 → 8, with the comment saying which file is the eighth.
- After *Upload assets to the release* and the count check, **before** *Publish
  the release*: fetch the asset list, then run `release-checksums.sh verify dist`.
  A failure leaves the draft, as RFC 037 intends.
- A re-run must stay idempotent: `write` regenerates the file from the
  downloaded artifacts, and `--clobber` replaces all eight together.

Bash, `set -euo pipefail`; `jq` (present on `ubuntu-latest`) for the JSON. No
new action, and no third-party download.

## 7. Required tests

The script, run locally against a scratch folder under `.git-exclude/tmp/`
(never anything outside the project):

- `write` on three files: the output passes `sha256sum -c`, is sorted, and names
  no directory.
- `write` on an empty folder fails.
- `verify` passes with a hand-made asset JSON that matches.
- `verify` fails, naming the file, for each of: one digest altered; one digest
  `null`; one asset missing; one extra asset on the release.
- **Against real data:** `verify` against 0.42.0's real asset list
  (`gh release view 0.42.0 --json assets`) and the one 0.42.0 asset already in
  `.git-exclude/fixtures/previous-release/` (with a `SHA256SUMS` written over a
  folder holding only that file). It must pass for that file and report the six
  absent ones as extra. That proves the field name and the format against GitHub,
  not against a guess.
- The workflow's YAML parses: `actionlint` if installed, or say it was not run.

## 8. Required documentation updates

- `docs/src/users/installation.md`, *Route 1*: download `SHA256SUMS` next to the
  archive, then run `sha256sum -c --ignore-missing SHA256SUMS` (Linux),
  `shasum -a 256 -c --ignore-missing SHA256SUMS` (macOS), or compare
  `Get-FileHash` with the line in the file (Windows). Say what it protects
  against and what it does not (§5, signing). **Confirm the macOS flag**: the
  architect is not sure every macOS `shasum` accepts `--ignore-missing`. Check it
  on the macOS runner (one `shasum --help` line in `native-smoke.yaml`'s log is
  enough, then remove it), or document the per-file comparison instead.
- `docs/src/dev/release.md`: a checklist item — *"the release has `SHA256SUMS`,
  and the `verify` step passed"*.

## 9. Acceptance criteria

- §7's tests observed.
- The workflow change confined to the `release` job and the asset count.
- The docs updated.
- **Proven at the next release** by the owner and the architect: the release shows
  eight assets, the `verify` step's log, and `sha256sum -c` passing on a
  downloaded archive. Recorded in the release evidence; the dev team's acceptance
  does not wait for it.

## 10. Prohibited shortcuts

- Treating a missing or `null` digest as a pass.
- Checking only the asset count, or only names.
- Publishing before `verify` has passed.
- Hard-coding asset names, or the number 8 anywhere but `EXPECTED_ASSET_COUNT`.
- Testing by pushing a tag. Tags are the owner's.

## 11. Relevant code or module boundaries

The `release` job of `.github/workflows/release-executable.yaml`, the new script,
two docs pages.

## 12. Compatibility constraints

`ubuntu-latest`'s GNU coreutils and `jq`. The file format is the one GNU
`sha256sum -c` and macOS `shasum -a 256 -c` both read.

## 13. Security constraints

- The script reads only the release's own asset list, through `gh` and the job's
  existing `GITHUB_TOKEN`. It needs no new permission.
- The public repository: no personal path in the script, the docs or the test
  output committed.
- Stated plainly in the user docs: this detects a corrupted or altered download,
  not a compromised release (§5).

## 14. Known risks

- **The `digest` field is recent on GitHub's side.** If it is ever missing,
  `verify` fails and the release stays a draft. That is the intended direction:
  a release that cannot be checked is not published silently. The owner can then
  decide.
- **A re-run that rebuilds artifacts** gives new bytes. `write` and `--clobber`
  keep the file and the assets consistent, and `verify` proves it.

## 15. Required evidence

### 15.1 Evidence plan

| Question | Level | Evidence |
|---|---|---|
| Does `write` produce a correct, checkable file? | logic | §7's `write` cases |
| Does `verify` refuse every kind of mismatch? | logic | §7's four failing cases |
| Does `verify` read GitHub's real format? | machine | the 0.42.0 run against the real asset list |
| Is the workflow well formed? | machine | `actionlint`, or recorded as not run |
| Does a real release carry it? | machine, at release | the next release (owner and architect) |

No GUI involved.

### 15.2 Deliverables

The rows above, observed, and the commit(s).

## 16. Review-request format

A package under `.git-exclude/review-request/` (working copy, not in the
repository) with a `## Entry point` section: the commit(s), the `git show`
commands, and full repo-relative paths, one per line. Report observed output; a
check not run is recorded as not run. You cannot push: report the commits, and
the architect pushes.

## 17. Handoff checklist

| Required item | Covered in |
|---|---|
| Task title | Heading |
| Purpose | §1 |
| Background | §2 |
| Applicable RFC and requirements | §3 |
| Change scope | §4 |
| Explicit non-change scope | §5 |
| Required implementation | §6 |
| Required tests | §7 |
| Required documentation updates | §8 |
| Acceptance criteria | §9 |
| Prohibited shortcuts | §10 |
| Relevant code or module boundaries | §11 |
| Compatibility constraints | §12 |
| Security constraints | §13 |
| Known risks | §14 |
| Required evidence | §15 |
| Required review-request format | §16 |
