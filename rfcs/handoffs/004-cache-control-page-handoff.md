# Handoff — RFC 004: Cache control page

**RFC.** [`rfcs/done/004-cache-control-page.md`](../done/004-cache-control-page.md)
**Shipped in.** v0.25.0
**Depends on.** RFC 003 (side-nav shell — makes adding a page trivial)

---

## 1. Implementation Handoff

### Goal
A third side-nav page (`NavPage::Cache`) giving per-directory visibility and
control over the embedding/thumbnail cache: a filterable summary table
(path, file count, total size, newest cached-at), per-row clearing, and a
form to cache an arbitrary directory on demand.

### Data source
Everything comes from `localcache` v0.20 — no new persistence:
| Need | Source |
|---|---|
| per-entry path | `EntryInfo::path` |
| per-entry size | `EntryInfo::metadata.file_size` |
| per-entry cached-at | `EntryInfo::updated_at` (unix secs) |

### Cache facade additions (`crates/cache`)
- A `DirCacheSummary` type.
- `summarize_by_dir()` on readers (uses `list_entries`, payload-free).
- `delete_in_dir()` on writers (removes db rows + thumbnail files,
  non-recursive).
- 4 spec tests.

### CachePage component (`crates/ui/main`)
Per-dir table with filter, refresh, per-row clear (🗑), add-dir form, and an
in-progress ⏳ row. `CacheRequire(Option<DirNode>)`: `None` = Explorer's
dir_node, `Some` = explicit Cache-page request. Single-task rule: the running
task handle is aborted on directory switch. `chrono` for local-time
formatting.

---

## 2. Task Breakdown / PR Plan

### PR 1 — Cache facade
1. `DirCacheSummary` + `summarize_by_dir` + `delete_in_dir` + 4 tests.
- **Reviewable in isolation:** `cargo test -p arama-cache`.

### PR 2 — Page + shell wiring
2. `NavPage::Cache` variant + nav rail icon button.
3. `CachePage` component (table, filter, refresh, per-row clear, add form,
   ⏳ row).
4. `CacheRequire(Option<DirNode>)` message + single-task abort-on-switch.
5. User docs (`docs/src/users/cache.md`).

---

## 3. Acceptance / QA Checklist

### Automated
- [ ] `cargo check --workspace` — zero errors, zero warnings.
- [ ] `cargo test -p arama-cache` — the 4 new `summarize_by_dir` /
      `delete_in_dir` spec tests pass.

### Manual — table
- [ ] The Cache page lists cached directories with path, file count, size,
      and a human-readable cached-at time.
- [ ] The filter narrows rows by path substring (case-insensitive).
- [ ] Refresh re-reads the cache and updates the table.

### Manual — actions
- [ ] Per-row 🗑 clears that directory's cache (db rows + thumbnails) and the
      row updates; other directories are untouched (non-recursive).
- [ ] The add-dir form caches an arbitrary directory; the ⏳ row appears
      while running and resolves to a real row on completion.
- [ ] Switching directory/page while a cache run is active aborts the run
      (single-task rule) without leaving a stuck ⏳ row.

### Regression
- [ ] Explorer-driven caching still works and appears on the Cache page.
- [ ] Wholesale "Cache delete" (Settings → File system) still works.
