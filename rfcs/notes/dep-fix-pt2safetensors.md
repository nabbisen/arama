# dep-fix-pt2safetensors

**Subject.** Build break in `pt2safetensors` 0.1.2 against `candle-core`
0.10 + `safetensors` ≥ 0.5; resolved in v0.35.0 by upgrading to
`pt2safetensors` 0.1.3.

---

## Root cause

`pt2safetensors` 0.1.2 (`nabbisen/pt2safetensors`) had two latent bugs that
surfaced together when resolved against current dependency versions:

### Bug 1 — `std` feature gating (safetensors ≥ 0.5)

```toml
# pt2safetensors 0.1.2 Cargo.toml
safetensors = { version = "0", default-features = false }
```

`serialize_to_file` has been gated behind `#[cfg(feature = "std")]` since
safetensors 0.5.0. With `default-features = false` and no explicit `features
= ["std"]`, the symbol is compiled out and the crate fails with:

```
error[E0432]: unresolved import `safetensors::serialize_to_file`
note: found an item that was configured out
    the item is gated behind the `std` feature
```

### Bug 2 — `View` trait version mismatch (candle-core 0.10)

`candle-core` 0.10 depends on `safetensors` **0.7**. `pt2safetensors` with
`version = "0"` resolved independently to `safetensors` **0.8**. Two
SemVer-incompatible copies of the crate entered the dependency graph, making
the `View` trait from one incompatible with the bound from the other:

```
error[E0277]: the trait bound `&candle_core::Tensor: View` is not satisfied
note: there are multiple different versions of crate `safetensors` in the
      dependency graph
```

## Fix — pt2safetensors 0.1.3

The upstream crate was patched and published as 0.1.3. Changes:

```toml
# [dependencies] — was:
candle-core = "0"
safetensors = { version = "0", default-features = false }
# now:
candle-core = "0.10"
safetensors = { version = "0.7", features = ["std"] }
```

Pinning `candle-core` to `"0.10"` prevents the `View` trait split.
Adding `features = ["std"]` restores `serialize_to_file`.

arama's `workspace.dependencies` updated to `pt2safetensors = "0.1.3"`.
