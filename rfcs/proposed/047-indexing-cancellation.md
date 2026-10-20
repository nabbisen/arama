# RFC 047: Indexing that can be stopped, and async that actually awaits

**Status:** Proposed
**Raised by:** the external audit of 0.41.2, 2026-09-01 (findings B2, C1, C4,
A7, D4), verified in
[`external-audit-2026-09-01-verification`](../notes/external-audit-2026-09-01-verification.md)
**Relates to:** [RFC 017](../done/017-visible-recoverable-error-ux.md),
[RFC 018](../done/018-ai-video-pipeline-resilience.md)

## Summary

**Four documents describe an application that can be interrupted. It cannot.**

`Task::abortable()` cancels a future by dropping it. A future with no `.await`
inside runs to completion in a single `poll()`, so dropping it does nothing. The
thumbnail phase of indexing has no await point, and neither does the
similar-pairs scan.

This is not a documentation error that got out of hand. It is **a design
intention written down and never checked against the implementation** — the
audit's phrasing, and it is the right diagnosis. `architecture.md` states the
intent explicitly, and the intent is sound; what is missing is the yield points
that would make it true.

## 1. What is actually true

**1.1 The thumbnail phase cannot be aborted.** `handle_cache_require`
(`app/src/core/update/cache.rs:30-56`) wraps `writer.upsert_all(requests)` — a
rayon fan-out over decode, resize, JPEG encode, and one ffmpeg subprocess per
video — in an `async move` block containing **no `.await` at all**. The
`.abortable()` handle is real and the Stop button dispatches to it; there is
simply no point at which the future can be dropped.

**The embedding phase is different, and the contrast is the evidence.** It
awaits (`image_embedding(...).await`, `:135`), so abort works there. Someone
solved this problem once and the fix did not reach the sibling phase.

**1.2 Three async bodies do blocking work.**

| Site | Blocking work inside `async` |
|---|---|
| `handle_cache_require` | `upsert_all` — rayon decode/resize/encode + ffmpeg per video |
| `SimilarPairsDialog::default_task` | `prepare_embeddings` — full cache read + O(N²) scoring |
| `download_generation` | `ensure_safetensors_in` — synchronous 605 MB conversion |

The third is the worst-behaved: it occupies a tokio worker for the length of a
605 MB file conversion **while a second model download runs concurrently**.

**The correct pattern is already in this repository.**
`publication.rs:22` and `sidecar/.../discovery/runtime.rs:45` both use
`tokio::task::spawn_blocking`. This is not a new technique to introduce; it is
one to apply consistently.

**1.3 The pair scan has no ceiling, no progress and no cancellation.** O(N²)
over every embedded file, in one synchronous block, reporting nothing until it
finishes. Exact search is a defensible choice at arama's scale — no recall loss,
no index to maintain — but it needs a bound and a way out, and it has neither.

**1.4 The Similarity Pairs button ungreys too early (A7).** It is gated on
`embedding_cached`, which means *"≥2 files in some directory"* and is set at
`cache.rs:96-97` — **roughly 35 lines before** the embedding task is spawned at
`:133`. So the button becomes available before any embedding exists, and
`using-arama.md` says twice that it stays greyed out until indexing finishes.

## 2. The four false claims

Verified against the implementation (audit D4):

| # | Document | Claim | Reality |
|---|---|---|---|
| 1 | `using-arama.md` | Pairs button greyed out until indexing finishes | §1.4 — ungreys before embedding starts |
| 2 | `using-arama.md` | "results appear progressively" | One `Task::perform`, one message, complete result |
| 3 | `using-arama.md` | Switching directories cancels the running index | True for embeddings, false for thumbnails |
| 4 | `cache.md` | "Pressing ◉ Stop aborts the run immediately" | Same as #3 |

**Three of the four are fixed in code, not prose** — they describe behaviour the
project intended and should have. Only #2 is a genuine documentation error, and
its correction is one sentence: *the scan runs in the background; results appear
when it completes.*

`similar_pairs_dialog.rs`'s own test comment already says
*"`prepare_embeddings` is `async fn` but never actually awaits anything"* — the
knowledge was in the tree, in a test, and never reached the design.

## 3. Design

**3.1 `spawn_blocking` for the three sites.** Follow `publication.rs:22`.
This alone fixes the scheduler-nesting and the blocked-worker problems, and it
does not fix cancellation.

**3.2 Cooperative cancellation for the indexing loop.** Chunk the work, yield
between chunks, and check a shared `AtomicBool` that the Stop button and the
directory switch both set.

`yield_now` alone is not sufficient and the distinction matters: dropping a
future at an await point cancels *the future*, but rayon work already dispatched
inside `upsert_all` continues. **The flag is what stops the work; the yield is
what lets the flag be observed.** Both are required.

**3.3 A bound, honest counts, and a progress signal for the pair scan.**

> **Moved here from [RFC 048](./048-library-scale.md) §3 on 2026-10-21**, with
> the owner's agreement. The two were halves of one user-visible behaviour: 047
> made the scan stoppable, 048 made its result honest. Shipping 047 alone would
> have produced a pair scan that stops cleanly and still misreports how much it
> found — a fix that satisfies the engineering half of the problem and leaves
> the user-facing half, which is the opposite of what the owner's second
> principle asks for. RFC 048 §3 now points here.

Images are silently capped at 50 (`similar_pairs_dialog.rs:33`,
`MAX_IMAGE_SIMILAR_PAIRS`); **videos have no cap at all** — verified again
2026-10-21, there is no video equivalent anywhere in the tree. Nobody chose that
asymmetry; it is what happens when a bound is added on one path.

Two problems, and the second is the one that matters:

1. **The cap is invisible.** A user with 300 similar pairs sees 50 and is told
   nothing.
2. **It is indistinguishable from completeness.** [RFC 036](../done/036-similarity-dialog-absence-states.md)
   built absence states precisely so "nothing found" could never be read as
   "something went wrong". A silent truncation reintroduces that ambiguity one
   level up: *"these are the similar pairs"* and *"these are the first 50 of the
   similar pairs"* render identically.

**A single explicit cap applied to both media types, with retained-versus-total
counts rendered in the dialog**, replaces a silent truncation with an honest
one. Raising or removing the cap is a separate question that depends on
RFC 048 §1 and is not answered here.

The scan must also be **interruptible** and must **report that it is running**.

**3.4 Gate the pairs button on completion, not on file count.** `embedding_cached`
should mean what its name says.

## 4. Why this is not urgent, and why it should not slip again

**Not blocking 0.42.0.** Nothing here loses user data or crashes. The cost is
CPU burned on abandoned work, a UI that misdescribes itself, and — with
[RFC 048](./048-library-scale.md)'s scale findings — an application that gets
slower to abandon the larger the library.

**But it interacts with a real hazard.** The audit's B3 records that media-path
ffmpeg invocations have no timeout and no output cap. A hung ffmpeg inside an
uncancellable phase is unkillable from the UI. The two findings are individually
moderate and jointly worse, and bounding the subprocesses belongs with this
work rather than after it.

## 5. Non-goals

- **Progressive result delivery.** Claim #2 is corrected in prose, not built.
  Streaming partial results is a larger design and nobody has asked for it.
- **Replacing exact search with an ANN index.** §1.3 argues exact search is
  right at this scale. It needs a bound, not an index.
- **The gallery's rendering cost.** [RFC 048](./048-library-scale.md).
- **Model fidelity.** [RFC 046](./046-audio-model-fidelity.md).

## 6. Acceptance

- Stop interrupts the thumbnail phase, demonstrated on a directory large enough
  to observe it.
- Switching directories mid-index cancels both phases.
- The three blocking bodies run under `spawn_blocking`.
- The pair scan reports progress and can be cancelled.
- **The pair cap applies to images and videos alike, and the dialog renders
  retained-versus-total counts** — a user can tell truncation from completeness
  (moved from RFC 048 §3).
- The pairs button reflects embedding completion.
- No document describes an interruption the code does not implement.

## 7. Decided — cancellation is cooperative and partial

**Was an open question; decided 2026-10-21** by the architect, under the owner's
standing principle that the end-state design should be chosen deliberately
rather than inherited from whatever the implementation happens to do. Owner may
overrule.

**Stopping mid-index leaves the cache populated for some files and not others,
and that is correct.** Reasons, in order of weight:

1. **It is what a user pressing Stop is asking for.** Stop means *stop doing
   more work*. Discarding work already finished is closer to Undo, which nobody
   asked for and which would make Stop expensive in proportion to how long the
   user waited before pressing it.
2. **Resuming is cheap and already works.** Re-running indexes only what is
   missing, so a partial index is a useful intermediate state rather than a
   corrupt one.
3. **"Transactional" could not be told truthfully here.** The cache spans two
   stores — SQLite rows and thumbnail files on disk. A rollback would have to be
   atomic across both to deserve the name. That is real machinery, and its
   failure mode is a half-rolled-back cache: strictly worse than a half-built
   one, and much harder to reason about.
4. **It preserves today's embedding-phase behaviour** rather than changing it.
   Transactional would be the larger change, not the smaller.

### 7.1 The condition this decision carries

Partial indexing is only acceptable **if the user can tell**. Under the owner's
second principle — users must not be confused or misled — a partially-indexed
directory that presents itself as fully indexed is a misdescription defect, and
this decision would create one if left bare.

**So it is binding on §3.4 rather than separate from it.** `embedding_cached`
must mean *embedding completed*, and a directory stopped midway must not satisfy
it. The pairs button staying greyed after a Stop is the correct, and visible,
consequence of a partial index — not a bug to work around.

**Added to acceptance (§6):** after a Stop, the application's own state
distinguishes *partially indexed* from *fully indexed*, and the UI does not
present the first as the second.

RFC 015's cache lifecycle does not cover cancellation; this section is where
that gap is now answered.
