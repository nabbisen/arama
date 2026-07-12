# RFC 022 - Image similarity search dependency strategy

**Status.** Implemented (Unreleased)
**Tracks.** Remaining audit-warning owner follow-up: decide whether to replace
`hnsw_rs` in image similar-pairs search so the transitive `bincode` 1.3 warning
can be removed.
**Touches.** `Cargo.toml`, `Cargo.lock`,
`crates/ai/src/pipeline/score/similarity/image.rs`,
`crates/ui/widgets/src/dialog/similar_pairs_dialog.rs`,
`rfcs/notes/audit-warning-burn-down.md`, `CHANGELOG.md`.

## Summary

`cargo audit` still reports `bincode` 1.3.3 as an allowed warning. The only
owner is:

```text
bincode 1.3.3 <- hnsw_rs 0.3.4 <- arama-ai
```

`hnsw_rs` is used in one function:
`crates/ai/src/pipeline/score/similarity/image.rs::find_similar_pairs`. That
function builds an approximate nearest-neighbor index, asks for a bounded number
of neighbors per image, then computes exact dot-product cosine scores before
returning pairs.

This RFC proposes a focused design before removing the dependency:

1. Prefer replacing `hnsw_rs` with an exact, bounded, parallel pairwise search
   if performance evidence is acceptable.
2. Keep a maintained ANN alternative as a fallback only if exact search is too
   slow for realistic galleries.
3. Preserve the user-visible similar-pairs contract: threshold filtering,
   no duplicate/self pairs, descending score order, and bounded result volume.
4. Remove `hnsw_rs` only after tests and dependency graph evidence show the
   replacement is correct.

## Why

This is the smallest remaining audit-warning owner with a local design surface:

- `localcache` owns `bincode` 2.0.1 and is the cache engine; replacement would
  reopen cache architecture.
- `paste` has many unrelated owners across Candle/gemm, tokenizers, image/rav1e,
  and UI/rendering paths.
- `ttf-parser` is inside the iced/font/rendering stack.
- `proc-macro-error2` currently has no active all-target owner in `cargo tree`.
- `hnsw_rs` is localized to image similar-pairs search.

The current HNSW use also has a subtle behavior contract: although the index is
approximate, returned scores are exact cosine dot products over L2-normalized
vectors. Replacing the candidate generator should not change scoring semantics,
but it may change recall, result count, ordering, and runtime.

## Design

### Part A - Search contract

The image similar-pairs function should expose this contract regardless of the
implementation strategy:

- skip empty inputs;
- never return self-pairs;
- never return both `(A, B)` and `(B, A)`;
- compute score as dot product over cached normalized CLIP vectors;
- include only scores greater than or equal to the user threshold;
- sort results by descending score;
- use a deterministic tie-breaker for equal or unordered floating-point scores;
- keep result volume bounded before handing pairs to the dialog.

The current `k_neighbors` argument is an implementation detail of HNSW. The
replacement should rename or reinterpret it as an output/candidate limit only
after documenting the new semantics.

### Part B - Preferred implementation: exact bounded search

The preferred implementation is an exact upper-triangle pairwise scan:

- iterate `i` over all image embeddings in parallel;
- compare only `j > i`;
- compute exact dot products;
- keep only threshold-passing pairs;
- bound retained pairs per worker or globally to avoid unbounded memory growth;
- merge and sort deterministically.

This removes the ANN dependency and gives complete recall for threshold-passing
pairs inside the retained result cap. It trades approximate indexing for
`O(n^2 * dimensions)` work. The implementation should include performance
evidence for realistic `n` values before review.

### Part C - Fallback: maintained ANN alternative

If exact search is too slow, the implementation may propose a maintained ANN
crate instead. That follow-up must document:

- dependency graph impact;
- Rust version and license;
- whether it introduces new audit warnings;
- how candidate count maps to the existing result contract;
- whether scores remain exact dot products after candidate generation.

Do not swap to a new ANN crate merely because it compiles. The replacement must
reduce audit risk without making the similar-pairs behavior less understandable.

### Part D - UI boundary

The similar-pairs dialog currently calls image search with `50` as a neighbor
count, then extends image pairs with video pairs. RFC 022 should not redesign
the dialog, but the implementation may rename the parameter or add a clear
constant so the dialog passes an output/candidate limit with explicit meaning.

The implementation should avoid returning an unbounded number of image pairs
for low thresholds.

### Part E - Audit note update

After implementation:

- remove the `hnsw_rs`/`bincode` 1.3 owner from
  `rfcs/notes/audit-warning-burn-down.md` only if `cargo audit` confirms the
  warning is gone;
- otherwise record why `hnsw_rs` was intentionally retained;
- do not add a new audit ignore.

## Touches in detail

### `crates/ai/src/pipeline/score/similarity/image.rs`

Replace or abstract the current HNSW candidate generator. Add focused unit
tests for pair generation, threshold filtering, duplicate avoidance, ordering,
and result limiting.

### `crates/ui/widgets/src/dialog/similar_pairs_dialog.rs`

Adjust the call site only if the search parameter is renamed or reinterpreted.
Keep dialog behavior otherwise unchanged.

### `Cargo.toml` and `Cargo.lock`

Remove `hnsw_rs` only after the search replacement compiles and tests pass.
The review package must show the resulting `cargo tree -i bincode@1.3.3`
outcome.

### `CHANGELOG.md`

Record the implementation as dependency/audit warning reduction and mention any
similar-pairs behavior change if result limiting semantics change.

## Non-goals

- No cache schema change.
- No CLIP model or embedding-shape change.
- No video similarity scoring redesign.
- No similar-pairs dialog layout redesign.
- No replacement of `localcache`.
- No new broad audit ignore.
- No release action or RFC lifecycle movement.

## Risks

- Exact pairwise search may be slower for large galleries. Mitigation: bound
  retained results, use parallel iteration, and capture performance evidence.
- Result counts may change because HNSW was approximate. Mitigation: document
  expected behavior and add fixture tests around deterministic ordering and
  limits.
- A new ANN dependency could simply move audit risk. Mitigation: prefer exact
  search first, and require dependency graph evidence for any alternative.
- Low thresholds can produce many matches. Mitigation: enforce a clear result
  cap before returning to the UI.

## Test plan

Required gates:

```sh
cargo fmt --all --check
cargo check --workspace
cargo test -p arama-ai
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo audit
```

Dependency checks:

```sh
cargo tree -i hnsw_rs
cargo tree -i bincode@1.3.3
cargo tree -i paste@1.0.15
```

Focused behavior tests:

- empty input returns no pairs;
- self-pairs and duplicate reverse pairs are not returned;
- threshold filtering is inclusive;
- output is sorted by descending score with deterministic tie behavior;
- result limit is honored;
- vectors with zero or mismatched lengths are handled explicitly or rejected by
  test-documented assumptions.

Performance evidence:

- include at least one synthetic benchmark-style timing or release-mode smoke
  measurement for a realistic number of 512-dimensional embeddings;
- record the host and command used;
- if exact search is too slow, stop and bring the maintained ANN alternative
  back for review instead of forcing the change.
