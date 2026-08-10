# RFC 037: Release publication atomicity

**Status.** Implemented (0.39.0). Accepted by the project owner 2026-08-10.

*Design question 1 was answered by observation before implementation, as
required:* `gh release edit --draft=false` does **not** detach a release from
its tag. The 0.37.0 hazard is specific to the opposite operation — editing an
already-published release into a draft.

*Verified by execution, not inference.* A deliberately broken upload on a
disposable tag produced exactly the 0.38.0 incident's shape — a release object
with zero assets — except invisible, badge-less, and with a red run. A second
run against the same tag reused that draft and completed it, publishing six
assets as a single release rather than duplicating it.

*Landed with it:* the notes-detection pipeline no longer pipes
`git for-each-ref` into `grep -q`, closing the `elif`-context EPIPE that could
silently substitute auto-generated notes for hand-written ones above the 64 KB
pipe buffer.
**Tracks.** Close a gap in [RFC 034](../done/034-release-workflow-reliability.md)'s
own guarantee: owner decision 2 — *nothing is published until everything has
succeeded* — is enforced **between** jobs but not **within** the `release` job,
where the release is created before its assets are attached.
**Touches.** `.github/workflows/release-executable.yaml` (`release` job only),
`docs/src/dev/release.md`. No product code.

**Why a new RFC rather than an edit to RFC 034.** RFC 034 is Implemented
(0.38.0) and its text is the record of what shipped. Editing it to describe
behaviour 0.38.0 did not have would falsify that record. This follows the
RFC 033 Part B → RFC 035 pattern: the gap is recorded where it was found, and
closed under its own number.

## Summary

RFC 034 established that a failed build must leave **no release** rather than a
release missing assets. That holds across jobs, via `needs: [build,
source-archive]`, and was verified twice.

It does not hold inside the `release` job. Its steps are:

```
create release  →  upload assets  →  verify asset count
```

`gh release create` publishes immediately, and nothing rolls it back. **Any**
failure after that step leaves a published release with missing or zero assets —
precisely the outcome decision 2 forbids, reached through a door one level below
where the guard was placed.

This RFC proposes creating the release as a **draft**, attaching and verifying
assets, and publishing only as the final action.

## Why now

This is not hypothetical. It happened, in production, on 2026-08-10.

Task 013 Step D pushed the real `0.38.0` tag. All five build variants and the
source archive succeeded. Inside `release`, the artifact download succeeded, the
release was **created and published**, and the upload step then failed with
`no matches found for dist/*` — because `actions/checkout` ran between download
and upload and cleared the untracked `dist/` directory.

The result was a published, non-draft `0.38.0` carrying the **Latest** badge
with **zero assets**: worse than 0.37.0, which at least shipped a source
archive. Recovery required deleting the release, deleting the tag, fixing the
step order, and re-tagging.

The step-order bug is fixed (`bb9b5b7`). **The structural gap that turned a step
failure into a public artifact is not.** Any future failure between creation and
a verified upload — a transient API error, a rate limit, a partial artifact
download, a bug in a step yet to be written — reproduces the same incident.

RFC 034's own handoff states the principle:

> *"Creating the release first and building afterwards cannot block — a late
> failure leaves a published release with partial assets, which is exactly what
> decision 2 forbids."*

The implementation applies that reasoning across jobs and then does the same
thing inside one.

## Current behaviour

`release` job, after `bb9b5b7`:

1. checkout
2. download all six artifacts into `dist/`
3. `gh release create` — **publishes**, or reuses an existing release
4. `gh release upload dist/* --clobber`
5. verify expected asset count

Steps 4 and 5 exist because uploads can fail independently of builds — RFC 034
§4.4 says so explicitly. The design already assumes this failure is possible; it
just places the public commit before it instead of after.

## Goals

- No failure inside the `release` job can produce a publicly visible release.
- The publish action is the **last** thing the job does, after the asset count
  has been verified.
- Re-runs stay idempotent, per RFC 034 Part B.
- No change to the trigger, the build matrix, the archive contracts, or the
  blocking gate between jobs.

## Non-goals

- Any change to what is built or how.
- Revisiting `types: [created, published]`. The `release:` trigger no longer
  exists; see Design.
- Rollback of a *published* release. Once published, correcting it is a
  human decision, not an automated one.
- crates.io publication.

## Design

```
create release AS DRAFT  →  upload assets  →  verify asset count  →  publish
```

A draft release is not publicly visible and does not take the "Latest" badge.
Every failure mode above therefore leaves nothing a user can see.

**Why this is available now, when RFC 034 rejected draft flows.** That rejection
concerned the **trigger**: under `on: release: types: [created, published]`, a
draft-then-publish release emits both events, running the matrix twice and
failing the second, non-idempotent upload. RFC 034 Part A removed the `release:`
trigger entirely — the workflow now listens only to `push: tags:` and
`workflow_dispatch`. There is nothing left to re-enter, so the objection no
longer applies.

**Idempotency.** The existing "reuse if it already exists" branch must also
recognise an existing **draft** from a prior failed run and reuse it, rather
than failing or creating a second release. With `--clobber` on upload, a re-run
then completes the sequence and publishes.

**Residual failure modes, all safe:**

| Fails at | Result |
|---|---|
| create | no release object at all |
| upload | a draft, invisible to users; re-run completes it |
| count check | a draft; the shortfall is visible in the run |
| publish | a draft; the assets are already correct |

The publish step is a single API call and is the only irreversible action. It
runs last, after every claim about the release has been checked.

**Orphan drafts** are the accepted cost. They are invisible publicly, visible to
maintainers, and reused by the next run.

## Design questions this RFC must settle

### 1. Does `--draft=false` detach the release from its tag?

0.37.0 established a real hazard: `gh release edit --draft` on an **already
published** release detaches it from its tag, rewriting its URL to
`releases/tag/untagged-<hash>`.

The proposed direction is the reverse — draft → published — and should not
behave that way. **It is adjacent enough that it must be verified rather than
assumed**, and it is the single largest risk in this RFC. If it does detach,
this design is unusable as written and the fallback is question 2.

### 2. Fallback if question 1 fails

If publishing a draft cannot be done safely, the alternative is to keep creating
published releases but **verify uploads before creating** — attach assets to
nothing is impossible, so this would instead mean deleting the release on
failure (`gh release delete`) as a compensating action. That is strictly worse:
it is a rollback rather than a commit-last design, and it briefly publishes. It
should be adopted only if question 1 rules out drafts.

### 3. Should the run fail loudly when it leaves a draft?

A draft left behind is a successful-looking run with no release. Recommendation:
**the job must fail**, so the run is red and the operator is told. A green run
that published nothing is the silent-failure shape this whole line of work
exists to remove.

## Relationship to Task 015

`.git-exclude/tasks/dev-team/015-release-notes-pipeline-latent-defect.md` fixes
a different defect in the same job: the notes-detection pipeline sits in an
`elif` condition, where `set -e` is suppressed, so an annotation larger than the
64 KB pipe buffer silently falls back to `--generate-notes` and ships an
auto-generated commit list in place of the hand-written notes.

Different mechanism, same family: **a step-level failure inside `release` that
produces a wrong outcome without turning the run red.**

They are separable, but they touch the same job and want the same review pass.
Recommendation: land them together, with Task 015 as the smaller, independently
verifiable half.

## Testing and verification

The `release` job only runs when `build` and `source-archive` both succeed,
which requires a real tag whose manifest matches it (Part F). It cannot be
exercised by `workflow_dispatch`.

**Proposed test — deliberately break the upload step:**

On a detached HEAD, bump the manifest to the next version, break the upload
step, tag, and push. Builds and archive pass; `release` runs; creation produces
a **draft**; upload fails; nothing is published. Confirm with `gh release list`
that no public release exists, then delete the draft, delete the tag, and let
the commit become unreachable. `main` never carries it.

This is the same technique Step C used, and it exercises exactly the property
being added.

**State the circularity honestly:** this test is only safe *because* of the
property it tests. If the change is wrong — if the release is created published
rather than draft — the test publishes an empty release. That risk is real but
strictly smaller than discovering the same defect during a genuine release, and
its recovery is now a known, rehearsed procedure.

**Also verify:** a second run against the same tag reuses the draft rather than
failing or duplicating, and completes it.

## Acceptance criteria

- No failure path inside `release` produces a publicly visible release.
- Publication is the job's final action, after asset-count verification.
- A run that ends with a draft **fails**, visibly.
- A re-run reuses an existing draft and completes it.
- Question 1 is answered by observation, not reasoning.
- `docs/src/dev/release.md` describes the draft-then-publish sequence, including
  what an operator should do if a draft is left behind.

## Risks

- **Question 1 is unresolved and could invalidate the design.** Named as the
  first design question for that reason.
- **Orphan drafts accumulate** if failures are frequent. Low impact; a
  maintainer-visible list, and each is reused rather than duplicated.
- **The test can publish if the change is wrong.** Stated above rather than
  minimised.
- **Over-engineering a channel that now works.** 0.38.0 shipped correctly. The
  counter-argument is that it shipped correctly on the *fourth* tag push, and
  the third one published a broken artifact through exactly this gap.

## Open questions

- Should this land before 0.39.0, or wait until there is other release-channel
  work to batch with it? It is not urgent — the proximate bug is fixed — but it
  is the difference between "this cannot happen again" and "this specific cause
  cannot happen again."
- Does `docs/src/dev/release.md` need an operator runbook for the orphan-draft
  case, or is the workflow comment sufficient?
