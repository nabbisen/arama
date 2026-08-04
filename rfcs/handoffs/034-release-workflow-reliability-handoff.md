# RFC 034 Handoff — Release workflow reliability

Companion to [RFC 034](../proposed/034-release-workflow-reliability.md), which
is **accepted for implementation** (owner, 2026-08-03) and stays in
`rfcs/proposed/` until the work ships, per
[RFC 000](../done/000-rfc-lifecycle-policy.md).

## 1. Design authority

1. [RFC 034](../proposed/034-release-workflow-reliability.md) — the governing
   design, including the two owner decisions and the workflow order they
   together require;
2. [RFC 030](../done/030-distribution-and-version-contracts.md) — the three
   distinct distribution contracts;
3. [RFC 012](../done/012-workspace-housekeeping.md) — the source-archive
   root-layout contract;
4. `docs/src/dev/release.md` — the authoritative release procedure, which the
   workflow must follow rather than reinterpret.

## 2. The two owner decisions are not negotiable

**CI creates the release.** Tag push triggers the workflow; the workflow
creates the release. Do not keep the `release:` trigger and merely add creation
on top — that reintroduces the event-type dependence this RFC exists to remove.

**A failed variant blocks the release.** Nothing is published until all five
variants have succeeded.

### The required order

```text
tag push
   │
   ▼
build all five variants        ← any failure stops here, nothing published
   │  (all succeeded)
   ▼
create the release
   │
   ▼
attach all assets              ← with --clobber
   │
   ▼
verify expected asset count
```

**This ordering is the whole point.** Creating the release first and building
afterwards cannot block — a late failure leaves a published release with
partial assets, which is exactly what decision 2 forbids.

The gate must be a **real job dependency** (`needs:`), not a best-effort
matrix. A matrix that continues past a failed variant defeats the decision.

**Failure leaves the tag with no release. That is correct** — a tag without a
release is visible and recoverable; a published release missing assets is
silent. Re-running after a fix creates it cleanly.

## 3. Already shipped — do not redo

Commit `8950240` during the 0.37.0 release:

- `types: [created]` → `types: [published]`;
- `workflow_dispatch` added.

Under Part A the `release:` trigger goes away entirely, so the first of these
is superseded rather than kept. **`workflow_dispatch` stays** — it is Part C's
pre-tag verification mechanism.

`[created, published]` was considered and rejected: a draft-then-publish
release emits both, running the matrix twice and failing the second upload.
Do not reintroduce it.

## 4. Required implementation

**4.1 Trigger.**

```yaml
on:
  push:
    tags: ['[0-9]+.[0-9]+.[0-9]+']
  workflow_dispatch:
```

The pattern must not match arbitrary tags. Arama uses `X.Y.Z` with no `v`
prefix (`docs/src/dev/release.md`).

**4.2 Source archive.** The workflow must produce the contract-compliant source
archive — files at archive **root**, no wrapping directory, excluding
`target/`, `.git/`, `.git-exclude/`, and `docs/book/`.

**GitHub's auto-generated source archives do not satisfy this** — they contain a
wrapping `<repo>-<sha>/` directory. Do not substitute them. Follow
`docs/src/dev/release.md`; if the workflow and the doc ever disagree, the doc is
authority and the workflow is wrong.

**4.3 Upload.** Add `--clobber` to `gh release upload`. This is what makes
re-runs safe; without it the first retry fails on existing assets and a small
failure becomes a public one.

**4.4 Asset verification.** After attaching, verify the expected asset count and
fail if it is short. "All variants built" and "all assets attached" are
different claims — an upload can fail after a successful build.

**4.5 `github.ref_name`.** Under `push: tags:` this is the tag name, which is
what the upload step already expects. Under `workflow_dispatch` against a
branch it is the branch name — so the dispatch path must **not** attempt to
upload. Dispatch is build-verification only (Part C).

**4.6 SemVer pre-release tags** (RFC 034 Part E, owner-accepted 2026-08-04).
Three pieces, all required together — implementing any one alone produces a
worse state than not implementing it:

- **E1** a second anchored trigger pattern, `'[0-9]+.[0-9]+.[0-9]+-*'`. Tag
  filters are **glob, not regex**; the hyphen must be required so the pattern
  cannot match arbitrary tags.
- **E2** detect the suffix and pass `--prerelease` to `gh release create`.
  GitHub does not infer prerelease status from the tag name, so without this a
  `-pre.1` release takes the "Latest" badge from the previous final version.
- **E3** the manifest version must equal the tag. `version.sh` runs for a
  pre-release exactly as for a final release, and the final release bumps again
  to strip the suffix. Two bumps per cycle is accepted.

**Do not** implement E1 without E2 and E3. A pre-release tag that publishes as
"Latest", or whose binaries report a different version than the release name,
is worse than having no pre-release support at all.

`docs/src/dev/release.md` must describe the pre-release path alongside the
final-release one, including the two-bump sequence.

## 4.7 Corrections from review 076 — apply before verifying

Both fix defects that would otherwise be found by spending a throwaway tag on a
workflow about to change:

- **Release notes.** `--generate-notes` loses the annotated tag's content. 0.37.0
  used `--notes-from-tag`, and its annotation carried the hand-written
  "Before you upgrade" block. Prefer the annotation, fall back only if absent.
- **`fail-fast`.** Set `fail-fast: false` on the build matrix. Blocking is
  enforced by the `needs:` relationship, not by matrix cancellation, so letting
  every variant report turns several tag-and-retry cycles into one.

## 5. Change scope

- `.github/workflows/release-executable.yaml`
- `docs/src/dev/release.md` — the procedure and checklist must match the
  implemented mechanism
- No product code, no manifest, no dependency change

## 6. Non-change scope

- The build matrix, targets, or CUDA variants.
- The source-archive contract itself — only who produces it.
- crates.io publication. Deferred at 0.37.0, out of scope here.
- Any attempt to diagnose RFC 034's cause 4 from inside the workflow.
- Any release, tag, or publication action beyond the verification in §7.

## 7. Verification — step 4 is the one that matters

1. **`workflow_dispatch` against a branch** — all five variants build. No
   upload attempted.
2. **Throwaway tag push** — release created, all assets attached.
3. **Re-run the same workflow** — upload succeeds rather than failing on
   existing assets.
4. **Deliberately break one variant** — confirm **no release is created** and
   the failure is visible.

Steps 1–3 prove the happy path. **Step 4 proves the property the owner actually
decided**, and this channel has only ever failed in ways the happy path would
not catch.

**Throwaway tag: `0.0.1`** — owner-accepted 2026-08-04, after the alternatives
were weighed and rejected. Verified free: arama's tag history starts at
`0.1.0`, so there is no collision with a real historical tag.

**Run the failure test before the success test.** If a broken variant behaves
correctly, **no release is ever created** — the public artifact is a tag and a
red Actions run. Running it first therefore minimises exposure and front-loads
the test most likely to reveal a design error.

**What deletion does and does not remove.** `gh release delete` plus a remote
tag delete removes the tag and the release. It does **not** remove the Actions
run entry, the repository event-feed records, or any clone fetched during the
window. The tag is volatile; the record is not. Keep the window short.

Cleanup, in this order, then confirm with `gh release list` and
`git ls-remote --tags origin`:

```sh
gh release delete 0.0.1 --yes          # only if a release exists
git push origin :refs/tags/0.0.1
git tag -d 0.0.1
```

**Do not** use `gh release edit --draft` for cleanup — on a published release it
detaches the release from its tag.

**The trigger pattern itself is unverified.** `workflow_dispatch` bypasses the
tag filter entirely, so it cannot validate the glob. The first tag push tests
two things at once — whether push-on-tag fires, and whether the pattern is
written correctly. If it does not fire, distinguish those before concluding
anything.

**A hazard found during 0.37.0:** `gh release edit --draft` on a published
release **detaches it from its tag** — the URL becomes
`releases/tag/untagged-<hash>`. It is recoverable, but do not use draft
toggling as a tidy-up mechanism. Delete the release outright, without
`--cleanup-tag` unless you also intend to remove the tag.

## 8. Acceptance criteria

- Tag push creates the release and attaches all five assets.
- A failed variant results in **no release**, visibly.
- Re-running is idempotent.
- The archive has files at root, correct exclusions, no wrapping directory.
- Asset count is verified after attachment.
- `workflow_dispatch` builds without attempting upload.
- `docs/src/dev/release.md` matches the implemented mechanism.
- Step 4 was actually executed, with its evidence.

## 9. Known risks

- **The one-way door moves to `git push --tags`.** Nothing in the
  implementation changes that; it is the accepted cost. Part C exists so a tag
  is not the first time a build is attempted.
- **The tempting shortcut** is to keep reacting to release events and bolt
  creation on. §2 forbids it.
- **Silent partial success.** If §4.4 is skipped because "the build passed",
  the channel keeps its ability to look successful while delivering nothing.

## 10. Required deliverables

A review request under `.git-exclude/review-request/NNN-<slug>/` opening with an
`## Entry point` section giving the pinned commit, the exact `git show` or
`git diff` command, and plain paths to every file. Report observed output; a
check not run is recorded as not run.
