# Cache Control

Click the **🗃** icon in the side nav to open the Cache page. It gives
per-directory visibility and control over the embedding/thumbnail
cache.

## The table

One row per directory that has cached files:

| Column | Meaning |
|---|---|
| Directory | Absolute path of the directory containing the cached files |
| Files | Number of cached files directly in that directory (images + videos) |
| Size | Sum of the cached files' sizes |
| Cached at | When the newest entry in that directory was cached (local time) |
| 🗑 | Clear this directory's cache |

Rows sort newest-first. The summary line below the table always shows
totals over **all** rows, regardless of the filter.

## Filtering

Type into the filter box to narrow the table to rows whose path
contains the typed text (case-insensitive). The **↻** button reloads
the table from the cache database.

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
counts, size, and timestamp.

Caching an already-cached directory is allowed and fast: unchanged
files are detected by metadata and skipped.

> **Note** — only one indexing run can be active at a time. Selecting
> a different directory in the Explorer cancels a Cache-page run (and
> vice versa), exactly as switching directories cancels an
> Explorer-initiated run.
