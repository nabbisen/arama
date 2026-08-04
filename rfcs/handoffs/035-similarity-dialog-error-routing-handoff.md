# RFC 035 Handoff — Similarity-dialog cache-error routing

Companion to [RFC 035](../proposed/035-similarity-dialog-error-routing.md),
which is **accepted for implementation** (owner, 2026-08-03) and stays in
`rfcs/proposed/` until the work ships, per
[RFC 000](../done/000-rfc-lifecycle-policy.md).

## 1. Design authority

1. [RFC 035](../proposed/035-similarity-dialog-error-routing.md) — the
   governing design and its four settled decisions;
2. [RFC 017](../done/017-visible-recoverable-error-ux.md) — the tier model this
   classifies against;
3. [RFC 033](../done/033-cache-dependency-and-rust-baseline.md) Part B — the
   deferral this closes, and the reasoning it applied one layer lower.

## 2. Decisions already made — do not re-open

From RFC 035, settled at proposal:

1. **Partial results are shown with a warning**, not discarded. A partial set
   still has value if the user is told it is partial.
2. **Errors render inline within the dialog**, not as toasts. A toast would
   outlive the dialog it describes and could appear over an unrelated screen.
3. **One aggregated message per dialog open**, not one per failed file —
   following RFC 032's one-warning-per-indexing-generation precedent.
4. **No curated variant allowlist.** Surface all failures uniformly.
   `Poisoned` is precisely the variant nobody predicted; a partial
   classification silently misses the next one.

### The two RFC-level open questions, settled here

RFC 035 left two questions marked implementation-level. Settling them is this
handoff's job, so the implementer does not have to guess:

- **Persist or auto-dismiss?** **Persist while the dialog is open.** Decision 2
  put the message inline precisely because it describes the content being
  rendered; a message that vanishes while the incomplete data it describes
  remains on screen recreates the silence this RFC removes.
- **Per-item failures, or only whole-lookup?** **Both, aggregated into the same
  single message.** They produce the same user-visible harm — a result set that
  is smaller than the truth — and splitting them would give two mechanisms for
  one problem.

## 3. Required implementation

Both dialogs — `crates/ui/widgets/src/dialog/similar_pairs_dialog.rs` and
`.../media_focus_dialog/similar_media.rs` — currently route every `CacheError`
to `eprintln!` and continue with `continue` (per item) or `return vec![]`
(whole lookup).

Replace that with:

- collect failures rather than discarding them;
- render one inline message per dialog open when any occurred;
- keep showing whatever results *were* obtained;
- keep the `eprintln!` diagnostic detail — the aggregate message is for the
  user, the per-item detail is for the developer.

**Behaviour must be identical between the two dialogs.** They are the same
problem in two places; divergence here is how the next inconsistency starts.

## 4. The distinction that must not be lost

**An empty cache is not a failure.** A user with no indexed data, or a
directory with no comparable media, must see the ordinary empty state — not an
error.

This is the same distinction RFC 033's empty-path fix established at the
embedding layer, where an empty input was treated as "no usable modality"
instead of vacuous success and produced a false error toast on first run. Do
not reintroduce that shape one layer up.

The message must describe the **effect** — some files could not be read — not
the mechanism, and must not appear when nothing failed.

## 5. Change scope

- `crates/ui/widgets/src/dialog/similar_pairs_dialog.rs` and its
  message/update/view files;
- `crates/ui/widgets/src/dialog/media_focus_dialog/similar_media.rs` and
  siblings;
- `app/src/core/update/` only if message routing requires it;
- `crates/i18n/src/{en,ja}.rs` — new strings;
- `docs/src/dev/testing.md` and
  `docs/src/dev/release-smoke-evidence-template.md` — one new smoke row.

## 6. Non-change scope

- Similarity scoring, thresholds, or ranking.
- `arama-cache` or its error types. RFC 033 established the cache layer
  propagates correctly; this is purely about consumption.
- The Cache page's existing `CacheLoadError` path — already correct.
- Any new error-UX mechanism. Classify using what RFC 017 already provides.
- Dialog layout beyond what surfacing requires.

## 7. Required tests and evidence

- **Aggregation:** N failures produce one message, not N.
- **Whole-lookup failure yields an error state, not an empty success.** This is
  the direct analogue of RFC 033's "`Poisoned` must never become a cache miss",
  one layer up, and it is the single most important assertion here.
- **Empty cache produces the ordinary empty state**, no error — §4.
- **Both locales resolve** the new keys, with a test. RFC 032's cycle showed a
  missing `ja` entry renders a raw key in the Japanese UI.
- **Rendered evidence** for both dialogs, using the method established during
  0.37.0: scratch-copy isolation under `.git-exclude/tmp/`, never the owner's
  real profile; native Wayland capture (`niri msg action screenshot-window`),
  window-scoped.

Add **`SMOKE-SIMILARITY-ERROR`** to `docs/src/dev/testing.md` and the evidence
template, and **execute it** as part of this work — the same correction applied
when `SMOKE-GALLERY-EMPTY` was added after an owner-found defect. A row added
but never run is how the gap recurs.

## 8. Lifecycle bookkeeping

RFC 033's Status field carries a placeholder deferral note instructing that it
be **replaced with the follow-up RFC's number once that RFC exists**. It now
exists. Update RFC 033's Status to reference RFC 035 by number in the same
change that ships this work.

## 9. Acceptance criteria

- Neither dialog presents a partial result as complete, nor a failure-induced
  empty result as an absence of matches.
- One aggregated inline message per dialog open, persisting while open.
- Per-item and whole-lookup failures share one mechanism.
- Behaviour identical between the two dialogs.
- Empty cache still shows the ordinary empty state.
- Both locales carry the strings, verified by test.
- `SMOKE-SIMILARITY-ERROR` added and executed.
- RFC 033's Status updated to name RFC 035.

## 10. Known risks

- **Over-surfacing.** Showing error text for an expected first-run empty cache
  would be a worse defect than the one being fixed. §4 is the guard.
- **Divergence between the dialogs.** They are near-identical today; implement
  once and share, rather than fixing each separately.

## 11. Required deliverables

A review request under `.git-exclude/review-request/NNN-<slug>/` opening with an
`## Entry point` section giving the pinned commit, the exact `git show` or
`git diff` command, and plain paths to every file. Report observed output; a
check not run is recorded as not run.
