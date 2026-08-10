# RFC 035: Similarity-dialog cache-error routing

**Status.** Implemented (0.38.0). Accepted by the project owner 2026-08-03; its
two open questions were implementation-level and were settled in the handoff.

*As built, two refinements beyond the handoff:* the unindexed-target branch
reports an error when a **batch** read partially failed, because the target's own
entry may be exactly what was silently dropped — a flat "no error" there would
have reproduced the defect this RFC removes. And `.flatten()` on the batch
cache-listing results, which discarded every per-entry `Err`, was replaced with
explicit tracking; it was the same defect hidden inside a batch call rather than
an explicit loop. Two absence states were knowingly left open and are carried by
[RFC 036](../done/036-similarity-dialog-absence-states.md).
**Tracks.** Close the deferral recorded in
[RFC 033](../done/033-cache-dependency-and-rust-baseline.md) Part B: the
similarity dialogs swallow every cache error and render a partial or empty
result with no user-visible signal.
**Touches.** `crates/ui/widgets/src/dialog/similar_pairs_dialog.rs`,
`crates/ui/widgets/src/dialog/media_focus_dialog/similar_media.rs`, their
message/update paths, `app/src/core/update/`, and `crates/i18n`.

## Summary

Both similarity dialogs route **every** `CacheError` variant to `eprintln!` and
then continue with whatever data they happened to collect. A user sees fewer
similar items — or none — and is told nothing. The result is indistinguishable
from "there genuinely are no similar files."

For a tool whose entire product is a similarity judgement, silently
under-reporting that judgement is the failure mode that matters most.

This RFC classifies those failures under
[RFC 017](../done/017-visible-recoverable-error-ux.md)'s tier model, which
already governs every other error surface in the application.

## Why now

RFC 033 adopted `localcache` 0.21.0 specifically because `0.20.1` could hand
back state a panicking thread had abandoned, and arama would present a
similarity score derived from it with no signal at any layer. The RFC's own
justification was that **a silent wrong answer is worse than a visible error**.

That reasoning was applied to the cache layer and stopped there. The dialogs
immediately above it still convert every error into silence. RFC 033 Part B
recorded the gap, deferred it as UX work outside a dependency task, and logged
it in RFC 033's Status field per RFC 000's deferred-work rule. This RFC is that
follow-up.

Two related facts make it worth doing now rather than later:

- the gap **predates RFC 033** and applies to every `CacheError` variant, not
  only the `Poisoned` one that RFC introduced;
- RFC 017's own first-pass classification table never covered these dialogs, so
  this is a genuine hole in that RFC's coverage rather than a regression.

## Current behaviour

`similar_pairs_dialog.rs` (image half; the video half is the same shape):

```rust
let lookup = match image_cache_reader.lookup(path) {
    Ok(lookup) => lookup,
    Err(err) => {
        eprintln!("failed to lookup image cache entry: {err}");
        continue;
    }
};
```

`media_focus_dialog/similar_media.rs` follows the identical pattern, with
either `continue` (per item) or `return vec![]` (whole lookup).

So there are two distinct silent outcomes:

| Shape | Effect |
|---|---|
| per-item `continue` | a similar file is **omitted** from the result set |
| whole-lookup `return vec![]` | the dialog reports **no similar media at all** |

The second is the more serious: an empty result is a positive claim that
nothing matched.

## Goals

- Every cache failure reaching a similarity dialog is either surfaced to the
  user or deliberately classified as a developer diagnostic, with the reason
  recorded.
- A partial result is never presented as a complete one.
- An empty result caused by failure is never presented as an empty result
  caused by absence.
- Behaviour is consistent between the two dialogs.

## Non-goals

- No change to similarity scoring, thresholds, or ranking.
- No change to `arama-cache` or its error types. RFC 033 already established
  that the cache layer propagates correctly; this is purely about consumption.
- No new error-UX mechanism. RFC 017's tiers and the existing toast and inline
  facilities are sufficient; this RFC classifies, it does not invent.
- No redesign of either dialog's layout beyond what surfacing requires.

## Design questions this RFC must settle

These are the decisions that made this UX work rather than a mechanical
routing change, and why RFC 033 correctly refused to make them in passing.

### 1. Partial versus empty

When some lookups fail and others succeed, is the right behaviour to:

- **(a)** show what succeeded, with a visible warning that the set is
  incomplete; or
- **(b)** treat the whole result as untrustworthy and show an error instead?

**Proposed: (a).** A partial similarity set still has value — the user can act
on the matches that were found — provided they are told it is partial. (b)
discards usable results to punish an incomplete one.

### 2. Toast versus inline

RFC 017 distinguishes blocking view errors, which render inline near the stale
or unavailable data, from recoverable action errors, which use app toasts.

Opening a similarity dialog is a **discrete user action**, which points at a
toast. But the dialog is also a **view rendering data**, which points at
inline. The Cache page precedent (`CacheLoadError`, inline) is the closer
analogue.

**Proposed: inline within the dialog**, because the error is *about the content
being rendered* and disappears with the dialog. A toast would outlive the
dialog it describes and could appear over an unrelated screen.

### 3. One message or per-item detail

A directory-wide failure could produce one error per file.

**Proposed: one aggregated message per dialog open**, following the precedent
RFC 032 set for indexing — one actionable warning per generation rather than
one per file. Per-item detail belongs in a developer diagnostic, not the UI.

### 4. Which variants are user-visible

Not every `CacheError` deserves the same treatment. A missing cache directory
on first run is expected; a poisoned pool is not.

**Proposed:** surface all failures uniformly rather than curating a variant
list. Curation requires predicting which variants matter, and RFC 033's
`Poisoned` case is precisely an example of a variant nobody predicted. A
uniform "similarity data could not be fully read" message with the variant in
the diagnostic is more honest than a partial classification that silently
misses the next new variant.

## Testing and verification

- Unit coverage for the aggregation logic: N failures produce one message, not
  N.
- A test that a whole-lookup failure yields an error state rather than an empty
  success — the analogue of RFC 033's "`Poisoned` must never become a cache
  miss", one layer up.
- Rendered evidence for both dialogs, per the smoke method established in the
  0.37.0 cycle: scratch-copy isolation, native Wayland capture, window-scoped.
- Localization keys present and resolving in **both** locales, with a test.
  RFC 032's release cycle showed a missing `ja` entry renders a raw key.

A new `SMOKE-SIMILARITY-*` row covering the failure path should be added to
`docs/src/dev/testing.md` and the evidence template, and executed as part of
the implementation — the same correction applied when `SMOKE-GALLERY-EMPTY` was
added after an owner-found defect.

## Acceptance criteria

- Neither dialog can present a partial result as complete, nor a
  failure-induced empty result as an absence of matches.
- One aggregated message per dialog open, not one per item.
- Behaviour is consistent between `similar_pairs_dialog` and
  `media_focus_dialog`.
- Classification follows RFC 017's tiers, with the chosen tier recorded.
- Both locales carry the new strings, verified by test.
- RFC 033's Status field is updated to reference this RFC by number, replacing
  its placeholder deferral note.

## Risks

- **Over-surfacing.** A user with an empty or partial cache on first run should
  not be met with error text for an expected condition. Mitigation: the message
  must describe the effect ("some files could not be read") rather than the
  mechanism, and must not appear when the cache is simply empty — the
  distinction RFC 033's empty-path fix already established at the embedding
  layer.
- **Scope creep into dialog redesign.** Mitigation: the non-goals fence this to
  classification and surfacing.

## Open questions

1. Should the partial-result warning persist while the dialog is open, or
   auto-dismiss? Inline placement implies persist; the existing toast
   infrastructure implies dismiss.
2. Does the same treatment apply to the media-focus dialog's *own* per-item
   failures, or only to the whole-lookup case? Proposed: both, aggregated
   together, but this doubles the surface and could be split.
