# Handoff — RFC 002: Replace the in-house cache engine with localcache

**RFC.** [`rfcs/done/002-replace-cache-engine-with-localcache.md`](../done/002-replace-cache-engine-with-localcache.md)
**Shipped in.** v0.23.0

---

## 1. Implementation Handoff

### Goal
Swap the in-repo `file-feature-cache` engine for the published `localcache`
crate, **keeping the `arama-cache` facade's public API stable** so the five
consumer sites need no change.

### Scope boundary (important)
- **Preserve:** `ImageCacheWriter` / `VideoCacheWriter` / `*CacheReader` and
  the `LookupResult` contract. Consumers (`app`, `arama-ai` ×2,
  `ui/widgets` dialogs ×2) are untouched in this RFC.
- **Rewrite:** only `arama-cache`'s internals, on top of `localcache`.
- **Delete:** `crates/engine/file-feature-cache/` (~1.5k lines + tests)
  at the end.

### Dependency changes
Drop `file-feature-cache`, `r2d2`, `r2d2_sqlite`, `sha2`. Add
`localcache = "0.19"`, `blake3 = "1"`, upgrade `rusqlite` to `0.39` (match
localcache's bundled version). Cache fingerprinting moves SHA-256 → BLAKE3.

### Data migration (breaking on-disk format)
The cache DB format changes. Provide a one-time migration
(`migrate_v1_if_present`) from the old DB path to the new
(`cache-v2.sqlite`); on failure the cache is rebuilt lazily, surfaced to the
user as a startup toast. Old and new paths differ in `env`.

---

## 2. Task Breakdown / PR Plan

### PR 1 — Rewrite facade internals on localcache
1. Add `localcache` / `blake3`, bump `rusqlite`; map writers/readers onto the
   localcache query/upsert API; keep the public types/signatures identical.
2. Keep the existing facade tests green; add coverage for the new internals.

### PR 2 — Migration + deletion
3. `migrate_v1_if_present` + startup toast on failure; new `cache-v2.sqlite`
   path in `env`.
4. Delete `crates/engine/file-feature-cache/` and its workspace entry; drop
   `r2d2*` / `sha2`.

---

## 3. Acceptance / QA Checklist

### Automated
- [ ] `cargo check --workspace` — zero errors, zero warnings.
- [ ] `cargo test -p arama-cache` — full facade test suite passes against the
      localcache backend.
- [ ] `Cargo.lock`: `file-feature-cache`, `r2d2`, `r2d2_sqlite`, `sha2`
      gone; `localcache`, `blake3` present; `rusqlite` at the matched version.

### Manual — behaviour parity
- [ ] Caching a directory produces embeddings + thumbnails as before; the
      focus view and pairs finder return the same matches.
- [ ] Re-running on an unchanged directory is fast (freshness detection /
      cache hit), not a full recompute.
- [ ] Modifying a file invalidates its cache entry and triggers recompute.

### Manual — migration
- [ ] With an old-format cache present, first launch migrates it to
      `cache-v2.sqlite`; on migration failure, the app shows a startup toast
      and rebuilds lazily (no crash, no data corruption).

### Regression
- [ ] The five consumer sites compile and behave unchanged (no API drift).
