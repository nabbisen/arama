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

**4.6 Pre-release support is withdrawn** (RFC 034 Part E, owner, 2026-08-08).

Part E was accepted 2026-08-04, implemented, and then found unbuildable:
internal deps carry `version = "0"` per RFC 030, and a Cargo requirement
without a pre-release component never matches a pre-release version. Any
pre-release fails at `cargo check`. See
`.git-exclude/reviewed/082-rfc034-e3-prerelease-semver-blocker-review.md`.

**Remove E1 and E2 from the workflow:**

- **E1** — the `'[0-9]+.[0-9]+.[0-9]+-*'` tag pattern and its comment. The
  final-version pattern stays.
- **E2** — the `PRERELEASE_FLAG` / `*-*)` suffix detection and its expansion in
  `gh release create`.

**Keep E3 — it is now Part F, and it is not pre-release-specific.** The
manifest-equals-tag check protects real releases exactly as much as candidates;
a `0.38.0` tag against a `0.37.0` manifest ships mislabelled binaries either
way. Leave the check in both the `build` and `source-archive` jobs, unchanged
in behaviour. Update its comment to reference Part F rather than Part E, so a
future reader does not delete it as Part E residue.

**Do not** attempt to make pre-releases work by loosening the internal
`version = "0"` requirements. That is RFC 030's contract and out of scope here.

`docs/src/dev/release.md` must describe the final-release path only. Its
"Pre-release tags" section goes — **except** the manifest-equals-tag rule inside
it, which is the doc's only statement of Part F and must be kept, restated for
final tags.

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

## 7. Verification — the failure test is the one that matters

> **The `0.0.1` authorization of 2026-08-04 is withdrawn** (owner, 2026-08-08).
> It was given for a plan that Part F makes unexecutable — a `0.0.1` tag against
> a `0.37.0` manifest fails every build leg. Do not push `0.0.1`. Its
> replacement is below.
>
> **Verification rides the real `0.38.0` release.** Approved in approach, owner,
> 2026-08-08. It does **not** authorize the release itself: the 0.38.0 cut needs
> its own go-ahead, as every cut has.

**Prerequisite: RFC 035 must ship first.** `rfcs/handoffs/035-similarity-dialog-error-routing-handoff.md`
is accepted and unstarted. 0.38.0's content is not complete until it lands, and
this verification is 0.38.0's release.

Step 1 is complete — run 30881426383 built all five variants green.

| # | Step | Tag | Manifest | Expected |
|---|---|---|---|---|
| 1 | `workflow_dispatch` on a branch | — | — | ✅ done: five variants build, no upload |
| 2 | **Failure test** | `0.38.0` | `0.38.0` | one variant broken → **no release created** |
| 3 | Success | `0.38.0` | `0.38.0` | release created, six assets, count verified |
| 4 | Idempotency | re-run step 3's workflow | unchanged | `--clobber` makes the re-run safe |

**Why a real version is safe here.** Under §2's required order a failed tag push
creates **nothing public** — no release, no assets, only a tag and a red run.
Step 2 behaving correctly therefore leaves nothing to clean up but the tag
itself. That is what makes spending `0.38.0` on the failure test acceptable, and
it is the same property being tested.

**Run the failure test first.** It front-loads the test most likely to reveal a
design error, at the point where nothing has been published.

### Keeping the deliberate break off `main`

The break must not land on `main`. Build it on a detached HEAD or a scratch
branch, tag that commit, and **push only the tag** — a tag can point at a commit
on no branch. After step 2, delete the tag and the commit becomes unreachable;
`main` never carries a break-then-revert pair.

```sh
git switch --detach main
# introduce the break, e.g. an invalid flag on one matrix variant
git commit -am 'TEMPORARY: break <variant> to verify RFC 034 Part D'
git tag -a 0.38.0 -m '...'
git push origin 0.38.0          # pushes the tag and its commit only
# observe: build fails, `release` job does not run, no release exists
git push origin :refs/tags/0.38.0
git tag -d 0.38.0
git switch main                 # the break commit is now unreachable
```

Then re-tag `main` proper for step 3.

**Confirm no release exists after step 2** with `gh release list` before
proceeding — "the run went red" is not the same claim as "nothing was
published."

**What tag deletion does and does not remove.** It removes the tag. It does
**not** remove the Actions run entry, the repository event-feed records, or any
clone fetched during the window. Here that costs nothing: a red run against a
real version is an ordinary event, not a record needing explanation.

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

- Tag push creates the release and attaches all six assets (five executables
  plus the source archive — `EXPECTED_ASSET_COUNT`).
- A failed variant results in **no release**, visibly.
- Re-running is idempotent.
- The archive has files at root, correct exclusions, no wrapping directory.
- Asset count is verified after attachment.
- `workflow_dispatch` builds without attempting upload.
- E1 and E2 are gone; the Part F manifest-equals-tag check remains in both jobs.
- `docs/src/dev/release.md` matches the implemented mechanism.
- **The failure test (§7 step 2) was actually executed**, with its evidence,
  including `gh release list` output showing no release was created.

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
