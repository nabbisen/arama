# Audit Warning Burn-down

This note records the July 2026 maintenance pass over `cargo audit` warnings.
It is not an RFC because no product behavior, architecture policy, or public
contract changed.

## Outcome

The pass resolved the warnings that had compatible patch releases available in
the current dependency graph:

- `anyhow` 1.0.102 -> 1.0.103
- `memmap2` 0.9.10 -> 0.9.11

After the initial lockfile update, `cargo audit` still exited successfully with
allowed warnings. These warnings are tracked below instead of being added as
broad new ignores.

After the RFC 022 image similarity implementation, the active audit surface is
down to three allowed warnings: `bincode` 2.0.1 through `localcache`, `paste`
1.0.15 through multiple transitive owners, and `ttf-parser` 0.25.1 through the
font/rendering stack.

After the RFC 024 image codec minimization implementation, the active image
dependency graph no longer reaches `ravif` or `rav1e`. `cargo audit` still
reports the same three allowed warning crates because `paste` remains reachable
through Candle/gemm, tokenizers, and Apple-target rendering paths.

After the RFC 027 audit-ledger refresh, the active audit surface is four
allowed warnings: `bincode` 2.0.1 through `localcache`, `paste` 1.0.15 through
multiple transitive owners, `rustybuzz` 0.20.1 through the SVG/font rendering
stack, and `ttf-parser` 0.25.1 through the font/rendering stack. `cargo audit`
exits successfully with these warnings under the current policy.

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

That modernization did not remove the remaining audit warnings.
`pt2safetensors` 0.1.3 still owns a transitive `candle-core` 0.10.2 path for
PyTorch to SafeTensors conversion, so Candle-related lockfile duplication is
not fully removed by the first-party Candle update.

## CLIP Source Strategy Follow-up

RFC 021 selected "retain runtime conversion" as the implementation outcome.
The currently pinned OpenAI CLIP revision publishes `pytorch_model.bin` and no
`model.safetensors` file in the inspected Hugging Face tree. Until arama has a
trustworthy pinned SafeTensors source or owner-managed mirror with provenance,
checksum, and embedding-regression evidence, `pt2safetensors` remains an
intentional dependency and the transitive Candle 0.10 path remains tracked.

## Image Codec Dependency Follow-up

RFC 024 switched `iced` from its codec-enabling `image` feature to
`image-without-codecs` and configured the workspace `image` dependency with
explicit codec features for arama's current accepted image formats: PNG, JPEG,
WebP, GIF, and BMP. The product image extension allowlist is unchanged.

`cargo tree -i ravif@0.13.0` and `cargo tree -i rav1e@0.8.1` now report no
active reverse dependency path. This removes the AVIF/ravif/rav1e owner path
from the `paste` warning, while leaving the remaining Candle/gemm, tokenizers,
and target-qualified rendering owners tracked below.

## Audit Ledger Refresh Follow-up

RFC 027 reconciled this note with the current `cargo audit` output. The refresh
does not change dependencies, add audit ignores, or introduce
`cargo audit --deny warnings`; it only records the current owner paths and
clarifies that allowed warnings still require rationale and revisit conditions.

## Event-listener Follow-up

`event-listener` 5.4.1 was flagged by `cargo audit` as RUSTSEC-2026-0221 (an
unsound `!Send` tag crossing thread boundaries via `StackSlot`), reaching
arama via `zbus` and the `async-*` desktop-portal stack behind `rfd` and
`file-handle` — not via `localcache`. This note never listed it: a Task 008
review request claimed all five then-current warnings were "already
tracked," and review 073 found that claim wrong, since this ledger has only
ever recorded the four below.

Task 010 Item A resolved it directly: `cargo update -p event-listener`
moved the locked graph to 5.4.2, published and unyanked, which does not
carry the advisory. `cargo audit`'s allowed-warning count went from five to
four as a result. No ignore, patch, or override was needed.

## Remaining Warnings

### `bincode` 2.0.1

Observed path:

```text
bincode 2.0.1 <- localcache 0.20.0 <- arama-cache
```

`localcache` 0.20.0 is the latest published crate version at the time of this
ledger refresh. `bincode` 3.0.0 exists, but arama reaches `bincode` through
`localcache`, not a direct workspace dependency. Burning down this warning
requires an upstream `localcache` release that moves off the unmaintained
`bincode` line, or a cache-engine design change.

### `paste` 1.0.15

Observed through multiple transitive owners, including:

- first-party `candle-core` 0.11 through `gemm`
- `pt2safetensors` 0.1.3 through transitive `candle-core` 0.10 and `gemm`
- `tokenizers` through `macro_rules_attribute`
- Apple-target rendering stack through `metal`, `wgpu-hal`, `wgpu`, and `iced`

This is not owned by a single direct dependency. Treat it as a dependency
modernization signal and revisit when direct dependency updates are planned.

### `rustybuzz` 0.20.1

Observed through the SVG/font rendering path:

```text
rustybuzz 0.20.1 <- usvg 0.45.1 <- resvg 0.45.1 <- iced_tiny_skia / iced_wgpu <- iced
```

This is target- and renderer-adjacent dependency risk rather than an arama
source-code dependency. Revisit when the iced rendering stack, `resvg`, or
`usvg` expose a compatible path away from the unmaintained `rustybuzz` line, or
when arama changes renderer/SVG support.

### `ttf-parser` 0.25.1

Observed through the font/rendering stack, including `fontdb`, `cosmic-text`,
`iced_wgpu`, `owned_ttf_parser`, `ab_glyph`, `usvg`, `resvg`, and `rustybuzz`
paths. `ttf-parser` 0.25.1 is the latest published crate version at the time of
this ledger refresh, so there is no compatible patch update to apply yet.

### `lru` 0.16.4 — added 2026-08-15

RUSTSEC-2026-0253, **unsound**: potential use-after-free from missing panic
safety in `LruCache::pop()`.

Observed path:

```text
lru 0.16.4 <- cryoglyph 0.1.0 <- iced_wgpu 0.14.0 <- iced_renderer 0.14.0 <- iced 0.14.0
```

Reached only through the iced rendering stack — no direct workspace dependency,
and no version of it that arama selects. Burning it down requires `cryoglyph`
or the `iced_wgpu` line to move to a patched `lru`. **Revisit when the iced
rendering stack next updates**, alongside `rustybuzz` and `ttf-parser`, which
enter through the same renderer and share that revisit condition.

Not previously listed because the advisory postdates this ledger's last
refresh; surfaced by the 0.39.1 release gate.

## Resolved by the snora 0.42.0 upgrade — 2026-09-03

**RUSTSEC-2026-0206, `rustybuzz` 0.20.1 — resolved by removal, not by a patch.**

The entry below recorded that `rustybuzz` reached arama only through
`usvg <- resvg <- iced_tiny_skia / iced_wgpu <- iced`, and set the revisit
condition as *"when the iced rendering stack next updates, alongside `ttf-parser`,
which enters through the same renderer."*

**That condition was met by Task 047** (`f248676`, snora 0.39.1 → 0.42.0). snora
0.42.0 stopped enabling `iced`'s `svg` feature transitively, which dropped
`resvg` and `usvg` from the graph — and `rustybuzz` with them. **It is no longer
in `Cargo.lock` at all** (verified: zero occurrences), so there is nothing left
to track.

`cargo audit` accordingly reports **four** allowed warnings, not five:
`bincode`, `paste`, `ttf-parser`, `lru`.

**`ttf-parser` did *not* go with it**, despite sharing the revisit condition — it
still enters through `fontdb`, `cosmic-text`, `ab_glyph` and `owned_ttf_parser`,
which are font paths independent of SVG. The shared revisit condition was
correct as a trigger and wrong as a prediction that both would resolve together.

*Noticed 2026-09-03 while preparing Task 044, not during Task 047's own review —
the upgrade's dependency reduction was reported in that package as a binary-size
benefit, and its effect on the advisory ledger went unremarked by both sides.*

## Resolved during a release gate — 2026-08-15

**RUSTSEC-2026-0257, `webbrowser` 1.2.1 — vulnerability, not a warning.**
Unix `BROWSER` handling allowed browser argument injection.

Unlike every entry above, this one was **directly actionable**: `webbrowser` is
a first-party dependency of `arama-ui-widgets`, the workspace pin is
`webbrowser = "1"`, and the fixed line (`>= 1.2.2`) is semver-compatible. Closed
by `cargo update -p webbrowser` (1.2.1 → 1.2.4) with no manifest change and no
other package moving in the lockfile.

Recorded here rather than only in the changelog because it is the first time
this project's release gate has caught a **vulnerability** rather than a
warning, and because the reason it was catchable — a direct dependency on a
permissive pin — is the distinction that separates it from the four entries
above that arama cannot act on alone.

## Existing Explicit Ignore

`.cargo/audit.toml` still has only the scoped `quick-xml` ignores added during
release-gate recovery. Those advisories enter through `wayland-scanner 0.31.10`;
the fixed `quick-xml` line requires a newer range than the current Wayland
scanner constraint accepts.
