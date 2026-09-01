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

**3.3 A bound and a progress signal for the pair scan.** The audit notes images
are already silently capped at 50 pairs while videos are unbounded (A8) — an
asymmetry nobody chose. A single explicit cap, with retained-versus-total counts
rendered in the dialog, replaces a silent truncation with an honest one. That
piece is [RFC 048](./048-library-scale.md)'s; what belongs here is that the scan
must be interruptible and must report that it is running.

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
- The pairs button reflects embedding completion.
- No document describes an interruption the code does not implement.

## 7. Open question

**Should cancellation be cooperative-and-partial, or transactional?** Stopping
mid-index leaves a cache populated for some files and not others. That is
already true today for the embedding phase, and it is probably correct — a
partial index is useful and resuming is cheap — but it has never been stated as
a decision, and RFC 015's cache lifecycle does not cover it.
