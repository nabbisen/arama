# Audit Warning Burn-down

This note records the July 2026 maintenance pass over `cargo audit` warnings.
It is not an RFC because no product behavior, architecture policy, or public
contract changed.

## Outcome

The pass resolved the warnings that had compatible patch releases available in
the current dependency graph:

- `anyhow` 1.0.102 -> 1.0.103
- `memmap2` 0.9.10 -> 0.9.11

After the lockfile update, `cargo audit` still exits successfully and reports
five allowed warnings. These warnings are tracked below instead of being added
as broad new ignores.

## Dependency Modernization Follow-up

RFC 020's implementation updated first-party Candle dependencies from 0.10 to
0.11 and the non-Linux sidecar `zip` dependency from 4.6.1 to 8.6.0. `zip`
8.6.0 declares Rust 1.88, which stays below arama's Rust 1.90 workspace floor.

The same five audit warnings remain after that modernization. `pt2safetensors`
0.1.3 still owns a transitive `candle-core` 0.10.2 path for PyTorch to
SafeTensors conversion, so Candle-related lockfile duplication is not fully
removed by the first-party Candle update.

## CLIP Source Strategy Follow-up

RFC 021 selected "retain runtime conversion" as the implementation outcome.
The currently pinned OpenAI CLIP revision publishes `pytorch_model.bin` and no
`model.safetensors` file in the inspected Hugging Face tree. Until arama has a
trustworthy pinned SafeTensors source or owner-managed mirror with provenance,
checksum, and embedding-regression evidence, `pt2safetensors` remains an
intentional dependency and the transitive Candle 0.10 path remains tracked.

## Remaining Warnings

### `bincode` 1.3.3

Observed path:

```text
bincode 1.3.3 <- hnsw_rs 0.3.4 <- arama-ai
```

`hnsw_rs` 0.3.4 is the latest published crate version at the time of this
maintenance pass. Burning down this warning requires an upstream `hnsw_rs`
release or a replacement/design change for the approximate-nearest-neighbor
dependency.

### `bincode` 2.0.1

Observed path:

```text
bincode 2.0.1 <- localcache 0.20.0 <- arama-cache
```

`localcache` 0.20.0 is the latest published crate version at the time of this
maintenance pass. Burning down this warning requires an upstream `localcache`
release that moves off the unmaintained `bincode` line, or a cache-engine design
change.

### `paste` 1.0.15

Observed through multiple transitive owners, including:

- first-party `candle-core` 0.11 through `gemm`
- `pt2safetensors` 0.1.3 through transitive `candle-core` 0.10 and `gemm`
- `tokenizers` through `macro_rules_attribute`
- `image`/`ravif` through `rav1e`
- the iced image/rendering dependency stack

This is not owned by a single direct dependency. Treat it as a dependency
modernization signal and revisit when direct dependency updates are planned.

### `proc-macro-error2` 2.0.1

`Cargo.lock` still contains `proc-macro-error2`, but no active path was found
with the normal all-target workspace tree check during this pass. Keep this on
the next audit pass and re-check whether a future lockfile refresh removes it
or reveals the active owner.

### `ttf-parser` 0.25.1

Observed through the font/rendering stack, including `fontdb`, `cosmic-text`,
`iced_wgpu`, `owned_ttf_parser`, `ab_glyph`, `usvg`, and `resvg` paths.
`ttf-parser` 0.25.1 is the latest published crate version at the time of this
maintenance pass, so there is no compatible patch update to apply yet.

## Existing Explicit Ignore

`.cargo/audit.toml` still has only the scoped `quick-xml` ignores added during
release-gate recovery. Those advisories enter through `wayland-scanner 0.31.10`;
the fixed `quick-xml` line requires a newer range than the current Wayland
scanner constraint accepts.
