# RFC 024 - Image codec dependency minimization

**Status.** Implemented (0.37.0)
**Tracks.** Remaining audit-warning owner follow-up: reduce unused image codec
dependency surface, including one `paste` owner path through the AVIF stack,
without changing arama's accepted image formats.
**Touches.** `Cargo.toml`, `Cargo.lock`, `env/src/file.rs`,
`crates/cache/src/core/thumbnail.rs`, `crates/cache/tests/**`,
`crates/ai/src/pipeline/encode/image/clip_encoder.rs`,
`rfcs/notes/audit-warning-burn-down.md`, `CHANGELOG.md`.

## Summary

`cargo audit` still reports `paste` 1.0.15 as an allowed warning with several
owners. One owner path is:

```text
paste 1.0.15 <- rav1e 0.8.1 <- ravif 0.13.0 <- image 0.25.10
```

The path exists because the workspace `image` dependency currently uses
default features, and `image/default-formats` includes AVIF. `iced`'s current
`image` feature also enables `image/default`.

arama's own current image allowlist is narrower:

```rust
["png", "jpg", "jpeg", "webp", "gif", "bmp"]
```

This RFC proposes a focused dependency-minimization pass:

1. Switch `iced` from the codec-enabling `image` feature to
   `image-without-codecs`.
2. Configure the workspace `image` dependency with explicit features for only
   arama's current accepted formats plus JPEG thumbnail output.
3. Remove unused AVIF/ravif/rav1e dependencies only if tests and dependency
   graph evidence confirm behavior remains correct.
4. Keep remaining `paste`, `bincode`, and `ttf-parser` warnings tracked.

## Why

This is not a full `paste` fix. `paste` remains reachable through Candle/gemm,
tokenizers, and possibly other rendering/image paths. But the AVIF branch is a
clear mismatch between accepted product formats and compiled codecs:

- arama does not scan AVIF files because `IMAGE_EXTENSION_ALLOWLIST` excludes
  `avif`;
- image thumbnails and CLIP preprocessing use `image::open` only for scanned
  image files;
- generated image thumbnails are JPEG files;
- removing unused codec stacks reduces compile/dependency/audit surface even
  when it does not fully clear the advisory.

The change has a product boundary: arama should continue accepting exactly the
same image extensions unless a separate RFC expands format support.

## Design

### Part A - Feature selection

The preferred implementation is:

```toml
iced = { version = "0.14", features = ["image-without-codecs", "tokio"] }
image = { version = "0.25", default-features = false, features = [
  "bmp",
  "gif",
  "jpeg",
  "png",
  "rayon",
  "webp",
] }
```

Rationale:

- `iced/image-without-codecs` keeps the iced image widget path available
  without forcing `image/default`.
- Explicit `image` features keep the codecs arama currently accepts.
- `jpeg` is required both for `jpg`/`jpeg` source files and generated `.jpg`
  thumbnails.
- `rayon` preserves the current `image` parallel-processing feature.

The implementation may adjust this list only with evidence from compile errors
or behavior tests.

### Part B - Format contract

The implementation must preserve the current image extension contract:

- PNG remains accepted.
- JPG/JPEG remains accepted.
- WebP remains accepted.
- GIF remains accepted.
- BMP remains accepted.
- AVIF remains not accepted unless a separate product-support decision is made.

No user-visible file type should be silently removed from the allowlist.

### Part C - Behavior surfaces

The implementation must check the surfaces that use decoded images:

- image thumbnail generation in `arama-cache`;
- CLIP preprocessing in `arama-ai`;
- generated thumbnail display through iced image widgets;
- gallery/focus/similar-pairs thumbnail display;
- cache page and cache maintenance behavior that relies on recorded thumbnail
  paths.

### Part D - Audit note update

After implementation:

- remove `ravif` / `rav1e` owner text from the `paste` note only if
  `cargo tree -i ravif` and `cargo tree -i rav1e` confirm absence;
- keep `paste` tracked if Candle/gemm, tokenizers, or UI owners remain;
- do not add new audit ignores.

## Touches in detail

### `Cargo.toml` and `Cargo.lock`

Change only image/iced codec feature selection. Expect lockfile pruning if
AVIF/ravif/rav1e dependencies become unreachable.

### `env/src/file.rs`

Should remain unchanged unless tests reveal the allowlist and enabled codecs no
longer match. Any allowlist change is product behavior and should come back for
review.

### `crates/cache/src/core/thumbnail.rs`

No functional change expected. Tests should verify thumbnail generation still
works for accepted formats and still writes JPEG thumbnails.

### `crates/ai/src/pipeline/encode/image/clip_encoder.rs`

No functional change expected. Tests or a focused smoke should verify accepted
formats still decode for CLIP preprocessing.

### `crates/cache/tests/**`

Add focused tests or fixtures for accepted image formats beyond JPEG if the
current suite does not cover them.

### `CHANGELOG.md`

Record this as dependency-surface minimization, not as new image format support.

## Non-goals

- No new image formats.
- No removal of currently accepted image formats.
- No change to CLIP model inputs, embedding shape, similarity scoring, cache
  schema, or thumbnail naming.
- No replacement of Candle, tokenizers, iced, fontdb, cosmic-text, or
  `ttf-parser`.
- No broad audit ignore.
- No release action or RFC lifecycle movement.

## Risks

- Iced thumbnail display could depend on `image/default` transitively.
  Mitigation: use `image-without-codecs` plus explicit workspace `image`
  features and run UI compile gates.
- An accepted format might lose decode support. Mitigation: add decode or
  thumbnail tests for PNG, JPEG, WebP, GIF, and BMP.
- Lockfile pruning could look larger than the code change. Mitigation:
  summarize removed codec dependencies in the review package.
- The `paste` warning may remain. Mitigation: state that this is owner-surface
  reduction, not full advisory removal, unless `cargo audit` proves otherwise.

## Test plan

Required default gates:

```sh
cargo fmt --all --check
cargo check --workspace
cargo test -p arama-cache
cargo test -p arama-ai
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo audit
```

Dependency checks:

```sh
cargo tree -i ravif
cargo tree -i rav1e
cargo tree -i paste@1.0.15
cargo tree -i ttf-parser@0.25.1
```

Focused behavior checks:

- image thumbnail generation for PNG, JPEG, WebP, GIF, and BMP;
- CLIP preprocessing decode smoke for the same accepted formats, if practical;
- generated JPEG thumbnail display still compiles through iced image widgets;
- AVIF remains outside `IMAGE_EXTENSION_ALLOWLIST`.
