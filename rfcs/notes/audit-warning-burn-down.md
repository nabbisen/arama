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

After the RFC 022 image similarity implementation, the active audit surface is
down to three allowed warnings: `bincode` 2.0.1 through `localcache`, `paste`
1.0.15 through multiple transitive owners, and `ttf-parser` 0.25.1 through the
font/rendering stack.

## Image Similarity Search Follow-up

RFC 022 replaced the localized `hnsw_rs` image similar-pairs candidate
generator with exact bounded pairwise search. This removed the `hnsw_rs` path
and the transitive `bincode` 1.3.3 audit warning. The image similar-pairs
dialog now asks for the top 50 image pairs globally instead of passing `50` as
an HNSW neighbor count.

The same lockfile refresh removed the previously stale `proc-macro-error2`
entry that had no active all-target workspace dependency path during the
maintenance pass.

## Cache Serialization Dependency Follow-up

RFC 023 selected retention for the implementation outcome. `localcache` 0.20.0
is still the latest published crate version observed in this pass, and both the
published crate metadata and the local upstream checkout keep `bincode` as an
unconditional dependency. Although `localcache` exposes `Codec::Json`, enabling
JSON in 0.20.0 would not remove the `bincode` audit warning by itself.

No cache payload codec, cache namespace, payload version, workspace patch, or
cache-engine replacement is included. Burning down the remaining `bincode` 2.0.1
warning still requires an upstream `localcache` release that can compile a
bincode-free cache path, or a separately reviewed cache-engine replacement.

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
