# Cache Control

Click the **🗃** icon in the side nav to open the Cache page. It gives
per-directory visibility and control over the embedding/thumbnail
cache.

Current arama versions use the v2 `localcache` database at
`.arama-cache/cache-v2.sqlite`. Legacy v1 cache databases are ignored
rather than imported; cached thumbnails and embeddings are rebuildable
when directories are indexed again.

## The table

One row per directory that has cached files:

| Column | Meaning |
|---|---|
| Directory | Absolute path of the directory containing the cached files |
| Files | Number of cached files directly in that directory (images + videos) |
| Media size | Sum of the indexed source files' sizes |
| Cached at | When the newest entry in that directory was cached (local time) |
| 🗑 | Clear this directory's cache |

Rows sort newest-first. The summary line below the table always shows
totals over **all** rows, regardless of the filter.

## Cache footprint and pruning

The Cache page also shows the actual cache footprint: the v2 SQLite
database, SQLite sidecar files, and generated thumbnails under
`.arama-cache/thumbnail/`. This is different from the table's media
size column, which describes source files rather than disk space used by
arama's cache.

Enter a one-off target in MiB and press **Prune** to remove reclaimable
cache data toward that target. Pruning removes orphan thumbnails first,
then removes oldest cached entries across image and video namespaces.
The source media files are never deleted.

SQLite may keep free pages inside `cache-v2.sqlite` after rows are
removed, so a prune target can be unreachable without database
compaction. In that case arama reports a partial prune and shows the
remaining unreclaimable footprint instead of treating it as a failure.

Pruning is explicit only: arama does not automatically prune on page
load, after indexing, or because free disk space is low.

## Filtering

Type into the filter box to narrow the table to rows whose path
contains the typed text (case-insensitive). The **↻** button reloads
the table from the cache database.

If the reload fails, the Cache page shows an inline error instead of
pretending the cache is empty. When previous rows are available, they
remain visible and are marked as stale until a reload succeeds.

## Clearing one directory

Press the **🗑** button on a row to remove every cached entry for that
directory — both the database records and the generated thumbnail
files. The files themselves are untouched; clearing only means the
next indexing pass will recompute thumbnails and embeddings.

Clearing is **not recursive**: subdirectories keep their own rows and
their own cache.

The clear buttons are disabled while a caching run is active.

## Caching a directory on demand

Type a directory path into the input at the top and press
**Cache this dir**. This runs the same indexing pipeline that
directory selection in the Explorer uses — thumbnails first, then AI
embeddings — without changing the Explorer's selected directory.

While the run is active:
- The directory's row shows **⏳ caching…** alongside a **◉ Stop**
  button in the Cached at column (a placeholder row appears if the
  directory had no cache yet).
- Pressing **◉ Stop** aborts the run immediately.
- The add and clear buttons are disabled (one run at a time).

When the run finishes, the table reloads and the row shows final
counts, media size, and timestamp.

Caching an already-cached directory is allowed and fast: unchanged
files are detected by metadata and skipped.

> **Note** — only one indexing run can be active at a time. Selecting
> a different directory in the Explorer cancels a Cache-page run (and
> vice versa), exactly as switching directories cancels an
> Explorer-initiated run.
