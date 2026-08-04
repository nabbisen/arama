# RFC 034: Release workflow reliability

**Status.** Proposed — accepted for implementation by the project owner
2026-08-03, with both open questions decided (see Resolved decisions). Remains
in `rfcs/proposed/` until the work ships, per RFC 000.
**Tracks.** Make the executable-asset release channel produce assets reliably,
or fail loudly, instead of silently producing nothing.
**Touches.** `.github/workflows/release-executable.yaml`,
`docs/src/dev/release.md`, and the release checklist. No product code.

## Summary

Arama's 0.37.0 release published its source archive and **no executable
assets**, with **no workflow run queued at all**. The failure was silent: the
release looked complete, and only an explicit check of the Actions run list
revealed that nothing had been built.

That is not a one-off. The release channel has at least four independent ways
to produce this same outcome, and every one of them looks identical to the
operator. This RFC makes the channel structurally resistant to three of them
and observable for the fourth.

Two fixes already shipped during the incident (commit `8950240`). This RFC
**documents them as shipped** rather than re-proposing them.

## Why now

The 0.37.0 release is the evidence. Sequence, as observed:

1. Release created already-published via `gh release create`. No run. The
   trigger was `release: types: [created]`, which did not fire.
2. Trigger changed to `types: [published]` and `workflow_dispatch` added,
   pushed to the default branch and verified live via the API.
3. Release deleted and recreated. No run.
4. Release toggled to draft and republished, re-emitting `published` an hour
   after the trigger change. No run.

The repository events API confirms **two** `ReleaseEvent … action=published`
entries for `0.37.0`. The workflow is `active`, repository Actions are enabled
with `allowed_actions: all`, and the `MSRV` workflow ran successfully on push
twice the same day. The event fires; the workflow does not start; the cause is
not visible from the API or CLI.

The channel is the mechanism by which all other work reaches users. It is
currently the least trustworthy part of the project.

## The four failure modes

| # | Cause | Operator sees |
|---|---|---|
| 1 | `created` does not fire for a release created already-published | release with no assets |
| 2 | A trigger change may not be live the instant its push completes | release with no assets |
| 3 | `workflow_dispatch` resolves the workflow from the *dispatched ref*, so a fallback added after a tag exists cannot rescue that tag's release | release with no assets |
| 4 | Unidentified — events fire, workflow does not start | release with no assets |

Causes 1 and 3 are structural. Cause 2 is a timing hazard. Cause 4 is unknown
and may be environmental; this RFC does not claim to fix it, but does require
that it stop being silent.

## Already shipped — documented, not proposed

Commit `8950240`, during the 0.37.0 release:

- **`types: [created]` → `types: [published]`.** `published` fires exactly once
  when a release becomes public, on both creation paths, and correctly never
  fires for a draft that is discarded.
- **`workflow_dispatch` added**, as a manual fallback.

`[created, published]` was considered and **rejected**: a draft-then-publish
release emits both, which would run the five-variant matrix twice and then fail
on the second `gh release upload`, since that step is not idempotent.

## Proposed design

### Part A — Tag-push-triggered release creation (primary)

Trigger the workflow on `push: tags:` and have it create the release, rather
than react to one.

```yaml
on:
  push:
    tags: ['[0-9]+.[0-9]+.[0-9]+']
  workflow_dispatch:
```

This removes causes 1–3 structurally, not by mitigation:

- **cause 1** disappears — there is no release event to match, and no
  `created`/`published` distinction to get wrong;
- **cause 2** disappears — the workflow that runs is the one contained in the
  pushed tag, so the tag and its build logic move together;
- **cause 3** disappears — the ref is a tag by construction, so
  `github.ref_name` is always a tag name and `workflow_dispatch` against that
  tag uses the same file.

**Costs, stated plainly.** The one-way door moves to `git push --tags`, which
is easier to do accidentally than creating a release. And the workflow must
reproduce arama's source-archive contract — files at archive root, no wrapping
directory, excluding `target/`, `.git/`, `.git-exclude/`, and `docs/book/`
(RFC 012, RFC 030) — because GitHub's auto-generated source archives contain a
wrapping directory and do **not** satisfy it.

The tag-name pattern must not match arbitrary tags. Arama uses `X.Y.Z` without
a `v` prefix (`docs/src/dev/release.md`).

### Part B — Idempotent upload

`gh release upload` currently runs without `--clobber`, so any re-run fails
once an asset exists. Add `--clobber`.

This is what makes recovery cheap: after any failure, re-running the workflow
must be safe. Without it, the only recovery is deleting assets by hand or
recreating the release, which is how a small failure becomes a public one.

### Part C — Pre-release verification

`workflow_dispatch` against a branch, before any tag exists, so all five asset
variants are proven to build before the irreversible step.

This is the mitigation for a risk recorded in the 0.37.0 release-readiness
report: neither shipped non-Linux target had been built since 0.36.2, and
`release-executable.yaml` fires only at release time, so a compile failure
would surface after the release existed.

Partially discharged by measurement during that release: all five
`#[cfg(windows)]` blocks in first-party code type-check under
`cargo check -p arama-sidecar --target x86_64-pc-windows-gnu`, and
macOS-conditional code is two `PathBuf::from` lines. Compile risk is low but
the full workspace has not been linked for `x86_64-pc-windows-msvc` or
`aarch64-apple-darwin` since 0.36.2.

### Part D — Fail loudly

The channel must not be able to produce nothing silently. Whatever mechanism is
chosen, the release process must include a step that **verifies assets exist**
after a release, and reports if they do not.

This is the only defence against cause 4, which this RFC cannot fix because it
cannot yet name it. A release with no assets is currently indistinguishable
from a successful one without manually inspecting the run list.

Minimum: add an explicit check to `docs/src/dev/release.md`'s checklist. Better:
a job step that fails when the expected asset count is not present.

## Non-goals

- No change to what is built — the five-variant matrix, targets, and CUDA
  variants are unchanged.
- No change to the source-archive contract itself, only to who produces it.
- No product code, dependency, version, or RFC-lifecycle change.
- No attempt to diagnose cause 4 from inside the workflow. If it recurs after
  Part A, that is new information and a separate investigation.
- No crates.io publication change. That channel was deliberately deferred at
  0.37.0 and is out of scope here.

## Compatibility and migration

- 0.37.0 remains without executable assets. This RFC does not retrofit it; the
  tag predates any fix, and cause 3 means no dispatch can rescue it.
- The first release under Part A is the proof. Until then the channel remains
  unverified, and the release checklist should say so.
- Existing tags are untouched.

## Testing and verification

Automated CI tests for CI are not proposed — the cost exceeds the value at this
project's scale. Verification is by exercise:

1. `workflow_dispatch` against a branch — all five variants build (Part C).
2. A tag push on a throwaway pre-release tag — the release is created and
   assets attached (Part A).
3. Re-run the same workflow — upload succeeds rather than failing on existing
   assets (Part B).
4. Deliberately break one variant and confirm the failure is visible rather
   than producing a silent partial release (Part D).

Step 4 matters most. Every other step proves the happy path, which is not where
this channel has failed.

## Risks

- **The one-way door moves to tag push.** Mitigated by the tag pattern, and by
  Part C making a pre-tag verification available. Not eliminated.
- **Reproducing the archive contract in CI could drift from the documented
  one.** Mitigated by keeping `docs/src/dev/release.md` as the authority and
  having the workflow follow it, not the reverse.
- **Cause 4 may recur under a `push` trigger too.** Part D is what makes that
  visible rather than silent. If it does recur, this RFC has still removed
  three of four causes and made the fourth detectable.

## Acceptance criteria

- The release channel produces assets from a tag push, or fails visibly.
- Re-running the workflow is idempotent.
- All five variants can be verified before a tag exists.
- A release with missing assets is detected by the release process rather than
  by chance.
- `docs/src/dev/release.md` matches the implemented mechanism.
- Shipped fixes from `8950240` are recorded as shipped, not re-proposed.

## Resolved decisions

Both decided by the project owner, 2026-08-03.

1. **The workflow creates the release.** Part A stands as written: a tag push
   triggers the workflow, and the workflow creates the release rather than
   reacting to one.
2. **A failed variant blocks the release.** Partial assets invite users to
   assume the rest are coming — the reason 0.37.0's Linux binary was
   deliberately not hand-attached.

### What the two decisions together require

They are not independent, and taken together they fix the workflow's shape.

If the release is created first and variants build afterwards, a late failure
leaves a **published release with partial assets** — which is exactly what
decision 2 forbids. Blocking is only achievable if nothing is published until
every variant has succeeded.

**Required order:**

```text
tag push
   │
   ▼
build all five variants        ← any failure stops here
   │  (all succeeded)
   ▼
create the release
   │
   ▼
attach all assets             ← --clobber, Part B
   │
   ▼
verify expected asset count   ← Part D
```

Consequences worth stating before implementation:

- **Failure leaves the tag without a release.** That is the intended outcome —
  a tag with no release is a visible, recoverable state; a release with missing
  assets is a silent, published one. Re-running after a fix creates the release
  cleanly.
- **The build gate must be a real dependency**, not a best-effort matrix. A
  matrix job that continues past a failed variant would defeat decision 2;
  `fail-fast` behaviour and an explicit needs-relationship between the build
  stage and the create stage are required.
- **Part D's asset-count check still applies** after creation, because "all
  variants built" and "all assets attached" are different claims — an upload
  can fail after a successful build.
- **Part C's pre-tag verification becomes more valuable, not less.** Under this
  shape a variant failure costs a whole tag-and-retry cycle, so proving the
  build first via `workflow_dispatch` avoids burning tags.

## Open questions

None. Both were resolved at acceptance.
