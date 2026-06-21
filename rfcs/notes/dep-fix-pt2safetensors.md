# dep-fix-pt2safetensors

**Subject.** Build break in `pt2safetensors` 0.1.2 against `candle-core`
0.10 + `safetensors` ≥ 0.5; temporary workspace patch applied in v0.35.0.

---

## Root cause

`pt2safetensors` 0.1.2 (`nabbisen/pt2safetensors`) has two latent bugs that
surface together when resolved against current dependency versions:

### Bug 1 — `std` feature gating (safetensors ≥ 0.5)

```toml
# pt2safetensors 0.1.2 Cargo.toml.orig
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
`version = "0"` resolves independently to `safetensors` **0.8**. Because both
are SemVer-incompatible minor versions, Cargo links two separate copies of the
`safetensors` crate. The `View` trait from 0.8 and the `View` trait from 0.7
are different types — so `candle_core::Tensor` (which implements 0.7's `View`)
does not satisfy the `serialize_to_file` bound from 0.8:

```
error[E0277]: the trait bound `&candle_core::Tensor: View` is not satisfied
note: there are multiple different versions of crate `safetensors` in the
      dependency graph
```

## Fix applied in v0.35.0

A local patched copy lives at `vendor/pt2safetensors/`. Changes from 0.1.2:

```toml
# before
safetensors = { version = "0", default-features = false }
# after
safetensors = { version = "0.7", features = ["std"] }
```

The root `Cargo.toml` routes `pt2safetensors` to this copy:

```toml
[patch.crates-io]
pt2safetensors = { path = "vendor/pt2safetensors" }
```

This pins `pt2safetensors` to the same safetensors minor version as
`candle-core` 0.10, eliminating both bugs simultaneously.

## Upstream fix required — pt2safetensors 0.1.3

The `vendor/` patch is a temporary workaround. The permanent fix is to
publish `pt2safetensors` 0.1.3 with the corrected dep declaration.

Minimal diff for that release (`Cargo.toml.orig`):

```toml
# [dependencies]
# before:
safetensors = { version = "0", default-features = false }
# after:
safetensors = { version = "0.7", features = ["std"] }
```

**When 0.1.3 is on crates.io:**

1. Delete `vendor/pt2safetensors/`.
2. Remove the `[patch.crates-io]` block from the root `Cargo.toml`.
3. Update `workspace.dependencies`: `pt2safetensors = "0.1"` (or `"0"`).
4. Run `cargo update pt2safetensors`.
5. Verify `cargo check --workspace` is clean.
6. Record the cleanup in CHANGELOG under the next release.
