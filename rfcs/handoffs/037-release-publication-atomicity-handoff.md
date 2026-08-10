# RFC 037 Handoff — Release publication atomicity

Companion to [RFC 037](../done/037-release-publication-atomicity.md), which
shipped in **0.39.0** and moved to `rfcs/done/` with that cut, per
[RFC 000](../done/000-rfc-lifecycle-policy.md).

## 1. Design authority

1. [RFC 037](../done/037-release-publication-atomicity.md) — the governing
   design;
2. [RFC 034](../done/034-release-workflow-reliability.md) — the guarantee this
   completes, and its two owner decisions, which are unchanged;
3. `docs/src/dev/release.md` — the authoritative procedure. If it and the
   workflow ever disagree, the doc is authority.

## 2. Phase 0 — answer design question 1 first. Do not skip this.

**`gh release edit --draft=false` must be proven not to detach a release from
its tag before anything else is written.**

0.37.0 established that `gh release edit --draft` on an *already published*
release detaches it, rewriting its URL to `releases/tag/untagged-<hash>`. The
proposed direction is the reverse and should be safe. **It is adjacent enough
that assuming it would be reckless, and if it detaches, this design is unusable
as written.**

This is cheap to establish and needs no release tag:

- create a throwaway **draft** release against an existing historical tag, or a
  scratch tag on a scratch commit;
- publish it with `--draft=false`;
- confirm the release still resolves at `releases/tag/<tag>` and its `tagName`
  is intact;
- delete it.

A draft is not publicly visible, so this costs nothing. **Report the observed
result before implementing.** If it detaches, stop and report — the fallback in
RFC 037 design question 2 is materially worse and needs its own decision.

## 3. Required implementation

Once Phase 0 passes, the `release` job's sequence becomes:

```
checkout → download → create AS DRAFT → upload --clobber → verify count → publish
```

**3.1 Create as draft.** `gh release create … --draft`. Notes handling is
unchanged — `--notes-from-tag` with the `--generate-notes` fallback stays as it
is (but see §5).

**3.2 Idempotent reuse must recognise drafts.** The existing "reuse if it
already exists" branch must find and reuse an existing **draft** from a prior
failed run, not fail on it and not create a second release. `gh release view`
does return drafts; confirm this rather than assuming. With `--clobber` on
upload, a re-run then completes the sequence.

**3.3 Publish last.** `gh release edit "<tag>" --draft=false` as the job's final
step, after the asset-count check has passed. Nothing may run after it.

**3.4 A run that ends with a draft must FAIL.** This is not optional. A green
run that published nothing is the silent-failure shape this entire line of work
exists to remove. If any step between create and publish fails, the job must be
red and the draft must remain for inspection.

**Do not** delete the draft on failure. It is the evidence, it is invisible to
users, and the next run reuses it.

## 4. What must not change

- The trigger, the build matrix, the archive contracts, the `needs:` gate.
- `EXPECTED_ASSET_COUNT` or what the count check asserts.
- The manifest-equals-tag check (RFC 034 Part F).
- Any product code.

## 5. Land Task 015 in the same change

`.git-exclude/tasks/dev-team/015-release-notes-pipeline-latent-defect.md` fixes
the `elif`-context EPIPE in the notes-detection pipeline: above the 64 KB pipe
buffer it silently falls back to `--generate-notes`.

Different mechanism, same family — a step-level failure inside `release` that
produces a wrong outcome without turning the run red. Same job, same review
pass. Land them together, with Task 015 as the smaller, independently
verifiable half.

## 6. Verification

**Phase 0** as above, reported before implementation.

**The main test — deliberately break the upload step.** On a detached HEAD, bump
the manifest to the next version, break upload, tag, push. Expected: builds and
archive green; `release` runs; a **draft** is created; upload fails; the job is
**red**; `gh release list` shows **no public release**. Then delete the draft,
delete the tag, and switch back so the commit is unreachable. `main` never
carries it.

**This test is only safe because of the property it tests.** If the change is
wrong — if the release is created published rather than draft — the test
publishes an empty release. That risk is real, is smaller than discovering the
same defect during a genuine release, and its recovery is now a rehearsed
procedure (Task 016). Proceed knowing it, and **stop immediately** if anything
public appears.

**Also verify:** a second run against the same tag reuses the draft and
completes it rather than failing or duplicating.

**Requires its own owner authorization** — it is a tag push on a knowingly
broken tree, the same class as Task 013's authorization 2. Ask before pushing.

## 7. Documentation

`docs/src/dev/release.md` must describe the draft-then-publish sequence, and
say what an operator should do when a run leaves a draft behind: inspect the
red run, fix, re-run against the same tag, and expect the draft to be reused
rather than duplicated.

## 8. Acceptance criteria

- Design question 1 answered **by observation**, reported.
- No failure path inside `release` produces a publicly visible release.
- Publication is the job's final action, after the count check.
- A run ending with a draft **fails**.
- A re-run reuses an existing draft and completes it.
- Task 015's fix landed and verified.
- `docs/src/dev/release.md` matches the implemented sequence.
- The break-upload test executed, with its evidence.

## 9. Known risks

- **Phase 0 can invalidate the design.** That is why it is Phase 0.
- **Orphan drafts accumulate** across failures. Accepted: invisible publicly,
  reused by the next run.
- **The temptation to delete the draft on failure** to keep things tidy. Do not
  — §3.4.

## 10. Required deliverables

A review request under `.git-exclude/review-request/NNN-<slug>/` opening with an
`## Entry point` section giving the pinned commit, the exact `git show` command,
and plain paths to every file. Include Phase 0's observed output, the run ID and
per-job conclusions for the break-upload test, and `gh release list` output
proving nothing was published. Report observed output; a check not run is
recorded as not run.
