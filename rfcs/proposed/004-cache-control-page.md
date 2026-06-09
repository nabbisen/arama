# RFC 004 — Cache control page

**Status.** Proposed
**Tracks.** A third side-nav page giving users per-directory
visibility and control over the embedding/thumbnail cache: a
filterable summary table, per-row clearing, and on-demand caching
of an arbitrary directory.
**Touches.** `crates/cache/` (two new facade methods + a summary
type), `app/src/core/` (new `NavPage::Cache` variant, page state,
messages, handlers, view), `crates/ui/main/` (new `CachePage`
component), `docs/src/users/` (new page docs), `rfcs/README.md`,
`CHANGELOG.md`.
**External design.**
[`docs/src/dev/design/004-cache-control-page-external.md`](../../docs/src/dev/design/004-cache-control-page-external.md)
describes the user-visible behaviour and view layout. This RFC covers
the internal design.

## Summary

The cache is currently invisible: users can only delete it wholesale
(Settings → File system → Cache delete) and can only populate it by
selecting a directory in the Explorer. This RFC adds a **Cache page**
(side-nav `NavPage::Cache`) with a per-directory table (path, file
count, total size, newest cached-at timestamp), substring filtering,
per-row clearing, and a form to cache any directory on demand. Caching
runs reflect in the table at start and finish.

The shell change is minimal by design — RFC 003 made the side-nav
extensible precisely so this page is "a new `NavPage` variant plus a
page component", with no structural rework.

## Data source

Everything the table needs already exists in `localcache` v0.20:

| Need | Source |
|---|---|
| Per-entry path | `EntryInfo::path` (canonical) |
| Per-entry size | `EntryInfo::metadata.file_size` |
| Per-entry cached-at | `EntryInfo::updated_at` (unix seconds, set on every write) |
| Cheap enumeration | `ReadPool::list_entries()` — returns `EntryInfo` without decoding payloads |

Grouping entries by their path's parent directory, and aggregating
count / size-sum / max(`updated_at`), is performed in the facade.

## Facade additions (`crates/cache`)

Two new public methods and one public type. Nothing existing changes,
so the RFC 002 compatibility contract is untouched.

### `DirCacheSummary` (new type, `types.rs`)

```rust
/// Per-directory aggregate of cached entries, for cache-management UIs.
#[derive(Debug, Clone)]
pub struct DirCacheSummary {
    /// Canonical path of the directory containing the cached files.
    pub dir_path: String,
    /// Number of cached files directly in this directory.
    pub file_count: usize,
    /// Sum of the cached files' recorded sizes, in bytes.
    pub total_size: u64,
    /// Newest `updated_at` among the entries (unix seconds).
    pub latest_cached_at: i64,
}
```

### `summarize_by_dir()` (new method on both readers)

```rust
impl ImageCacheReader {            // and VideoCacheReader
    /// Group all cached entries by their parent directory and aggregate
    /// count, size, and the newest cached-at timestamp per directory.
    pub fn summarize_by_dir(&self) -> Result<Vec<DirCacheSummary>>;
}
```

Implementation: `self.read.list_entries()?` → fold into a
`BTreeMap<PathBuf, (usize, u64, i64)>` keyed by `path.parent()` →
map into `Vec<DirCacheSummary>`. No payload decoding; cost is one
indexed table scan per namespace.

The **app** merges the image-reader and video-reader summaries (same
directory key → sum counts/sizes, max timestamps), because "the cache
for a directory" is the union of both namespaces from the user's point
of view. Merging lives in the app (page component), not the facade,
to keep the facade per-namespace like every other API it has.

### `delete_in_dir()` (new method on both writers)

```rust
impl ImageCacheWriter {            // and VideoCacheWriter
    /// Remove every cached entry whose file lives directly in `dir`,
    /// deleting the associated thumbnail file (when recorded) as well.
    /// Returns the number of entries removed.
    pub fn delete_in_dir(&self, dir: &Path) -> Result<usize>;
}
```

Implementation: query the directory's entries **with payloads**
(`query_run(|q| q.path_in_dir(dir, false))` on the read pool — the
payload carries `thumbnail_path`), then for each entry:
`std::fs::remove_file(thumbnail)` best-effort, then
`self.write.remove(path)`. Thumbnails are removed best-effort (a
missing thumbnail file is not an error); database removal errors abort
with the count removed so far reported in the error context.

Non-recursive (`recursive = false`) by design: one table row is
exactly one directory; subdirectories have their own rows.

## App-side design

### State

```rust
// app/src/core.rs
pub(crate) enum NavPage { Explorer, Cache, Settings }   // + Cache

pub struct App {
    // ...existing fields...
    cache_page: CachePage,    // new persistent page component
}
```

`CachePage` (new component, `crates/ui/main/src/views/cache_page/`,
following the existing `views/` component pattern with
`message.rs` / `update.rs` / `view.rs` submodules):

```rust
pub struct CachePage {
    /// Merged per-directory rows, sorted newest-first. Loaded lazily.
    rows: Vec<DirRow>,
    /// Substring filter (case-insensitive). Empty = show all.
    filter: String,
    /// Path input of the add-directory form.
    dir_input: String,
    /// Directory of the active caching run, when one is in flight.
    /// Drives the ⏳ row indicator and disables clear/cache buttons.
    active_run: Option<PathBuf>,
    /// True while a row-clear or table reload task is in flight.
    busy: bool,
}

struct DirRow {
    dir_path: String,
    file_count: usize,
    total_size: u64,        // bytes
    latest_cached_at: i64,  // unix seconds
}
```

### Messages

```rust
// cache_page::message
pub enum Message {
    Event(Event),
    Internal(Internal),
}

pub enum Event {
    /// Ask the app to start the indexing pipeline for this directory.
    CacheRequest(PathBuf),
    /// Ask the app to clear this directory's cache rows.
    ClearRequest(PathBuf),
}

pub enum Internal {
    FilterInput(String),
    DirInput(String),
    RefreshPressed,
    /// Result of the async table load.
    RowsLoaded(Vec<DirRow>),
}
```

App-level (`app/src/core/message.rs`):

```rust
pub enum Message {
    // ...existing...
    CachePageMessage(cache_page::message::Message),
    /// Async per-row clear finished (count removed or error string).
    CacheClearFinished(Result<usize, String>),
}
```

### Message flow

```
                      [Cache this dir]                [🗑 row]
                            │                            │
                Event::CacheRequest(dir)      Event::ClearRequest(dir)
                            │                            │
              App: validate dir exists        App: Task::perform(
                  set active_run                 delete_in_dir on both
                  reuse CacheRequire             writers, CacheClearFinished)
                  pipeline with an                       │
                  explicit dir override                  ▼
                            │                  CacheClearFinished
                            ▼                    → reload table
            ThumbnailCacheFinished /
            EmbeddingCacheFinished
              → clear active_run
              → reload table
```

### Reusing the indexing pipeline

Today `Message::CacheRequire` reads `self.dir_node` (the Explorer's
selection). The Cache page must index an arbitrary directory *without*
changing the Explorer's selection. The smallest change that supports
both callers:

```rust
// message.rs
CacheRequire(Option<DirNode>),   // None = use self.dir_node (Explorer)
                                 // Some = explicit target (Cache page)
```

- All existing senders pass `None` — a mechanical, three-site edit.
- The Cache page handler builds a `DirNode` for the requested path
  with the same `dir_node(path, &target_media_type)` helper `App::new`
  already uses, and passes `Some(node)`.
- `ThumbnailCacheFinished` / `EmbeddingCacheFinished` stay unchanged
  except for two additions: clear `cache_page.active_run` and trigger
  a table reload when the Cache page has been loaded at least once.
- The existing single-task rule (`task_handle` + abort-on-dir-switch)
  applies as-is: a Cache-page run and an Explorer run can never overlap
  because both go through the same `CacheRequire` → `task_handle` path.
  An Explorer directory switch aborts a Cache-page run, identically to
  how it aborts an Explorer run; `active_run` is cleared in the same
  `on_dir_changed` site that aborts.

This is deliberately the *only* touch to the pipeline. No new task
types, no second handle, no run queue.

### Table loading

Loading is async (`Task::perform`) because `summarize_by_dir` scans
the database:

```rust
async {
    let img = ImageCacheReader::onetime(loc)?.summarize_by_dir()?;
    let vid = VideoCacheReader::onetime(loc)?.summarize_by_dir()?;
    merge(img, vid)   // sum counts/sizes, max timestamps, per dir key
}  → Internal::RowsLoaded(rows)
```

Triggered on: first navigation to the Cache page, Refresh press,
run start, run finish, and clear finish. While loading, `busy = true`
disables the Refresh button (the table keeps showing the previous
rows — no flicker to empty).

### View

Implements the external design's layout: add-directory form, filter
row, the table as an iced `column` of styled `row`s inside a
`scrollable`, and the summary line. Formatting helpers:

- `human_size(u64) -> String` — `41.2 MB` style, binary-1024 units.
- `format_timestamp(i64) -> String` — local `YYYY-MM-DD HH:MM` via the
  `chrono` crate (new workspace dependency; tiny, ubiquitous, and the
  only reasonable way to render local time).

Buttons disabled per the external design: the add button and every 🗑
while `active_run.is_some()`; Refresh while `busy`.

## Dependency changes

| Crate | Change |
|---|---|
| `chrono` | added to workspace deps, `default-features = false, features = ["clock"]` — local-time formatting in the table |

No other additions. The facade methods use only `localcache` APIs
already in the tree.

## Touches in detail

| File | Change |
|---|---|
| `crates/cache/src/types.rs` | Add `DirCacheSummary` |
| `crates/cache/src/core/image.rs` | `ImageCacheReader::summarize_by_dir`, `ImageCacheWriter::delete_in_dir` |
| `crates/cache/src/core/video.rs` | Same pair on the video handles |
| `crates/cache/src/lib.rs` | Re-export `DirCacheSummary` |
| `crates/cache/tests/integration_tests.rs` | New spec tests (below) |
| `crates/ui/main/src/views/cache_page/` | New component (state, message, update, view) |
| `app/src/core.rs` | `NavPage::Cache`; `cache_page` field |
| `app/src/core/message.rs` | `CachePageMessage`, `CacheClearFinished`; `CacheRequire(Option<DirNode>)` |
| `app/src/core/update.rs` | New handlers; 3 mechanical `CacheRequire(None)` edits; reload hooks in the two `*Finished` handlers |
| `app/src/core/view.rs` | Third nav-rail button; `NavPage::Cache` body arm |
| `docs/src/users/cache.md` + `SUMMARY.md` | User documentation |
| `docs/src/dev/design/004-…-external.md` | External design (this RFC's companion) |

## New tests (cache crate)

Validating the design spec, not the code:

1. `summarize_by_dir_groups_and_aggregates` — three files in dir A,
   one in dir B → two summaries with correct counts, size sums, and
   `latest_cached_at` equal to the newest entry's timestamp.
2. `summarize_by_dir_empty_cache` — returns an empty vec, no error.
3. `delete_in_dir_removes_entries_and_thumbnails` — entries in the
   target dir are gone (lookup → Miss), thumbnail files removed from
   disk, sibling-directory entries untouched, returns the removed count.
4. `delete_in_dir_is_not_recursive` — entries in a subdirectory
   survive a parent-directory clear.

## Risks

| Risk | Mitigation |
|---|---|
| `list_entries` scans grow with cache size | `EntryInfo` skips payload decode; tens of thousands of rows scan in milliseconds. Revisit with pagination only if real usage shows a problem |
| Clear racing a caching run | UI disables 🗑 while `active_run.is_some()`; additionally writers serialize on the localcache write connection, so a race degrades to ordering, not corruption |
| Explorer dir-switch aborting a Cache-page run surprises the user | Acceptable and documented: one indexing task at a time is an existing, visible invariant (processing spinner). A run queue is out of scope |
| `chrono` dependency creep | `default-features = false`; only `clock` enabled |

## Alternatives considered

- **Per-file table rows.** Rejected for v1: directory granularity
  matches both the clearing operation users want and the data volume a
  table can present; per-file drill-down can layer on later without
  schema changes.
- **Facade-level namespace merging** (one `summarize_by_dir` over both
  namespaces). Rejected: every existing facade API is per-media-type;
  merging is a presentation concern and stays in the page component.
- **A run queue for multiple cache requests.** Rejected: the
  single-task invariant is load-bearing throughout `update.rs`
  (abort-on-switch, processing flag). A queue is a separate RFC if
  demand appears.
- **Storing per-directory aggregates in the database.** Rejected:
  derived data; the group-by is cheap and always consistent.

## Open questions

1. Should the Cached at column render relative time ("2 days ago") or
   absolute local time? The design says absolute (`YYYY-MM-DD HH:MM`);
   relative time needs periodic re-render and adds little. Default:
   absolute, revisit on feedback.
2. Should an Explorer-initiated run also be cancellable from the Cache
   page (a stop button on the ⏳ row)? Deferred — not in the requested
   scope; trivially added later since the abort handle already exists.
