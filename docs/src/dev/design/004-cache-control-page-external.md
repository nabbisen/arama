# External Design — Cache Control Page

Companion document to RFC 004. This describes the user-visible
behaviour and view structure; RFC 004 covers the internal design.

## Purpose

Give users visibility into and control over the embedding/thumbnail
cache, per directory:

1. **See** which directories hold cached files — when they were cached,
   how many files, and how much payload they occupy.
2. **Filter** the table by directory path substring.
3. **Clear** the cache for one directory (one table row).
4. **Request** caching of a specific directory, and watch the table
   reflect the run starting and finishing.

## Navigation

A third icon is added to the side-nav rail:

```
┌─────┐
│ 📁  │  Explorer
│ 🗃  │  Cache      ← new (lucide: database)
│ ⚙  │  Settings
└─────┘
```

The order places Cache between Explorer and Settings: it is a working
page (like Explorer), not a configuration page.

## Page layout (text mock)

```
┌──────┬──────────────────────────────────────────────────────────────┐
│      │  Cache                                                       │
│  📁  │                                                              │
│ [🗃] │  ┌ Add directory to cache ───────────────────────────────┐  │
│  ⚙  │  │ [ /path/to/directory…            ] [ Cache this dir ] │  │
│      │  └────────────────────────────────────────────────────────┘  │
│      │                                                              │
│      │  Filter: [ holidays…              ]          ↻ Refresh      │
│      │                                                              │
│      │  ┌──────────────────────────┬───────┬─────────┬──────────┬─┐│
│      │  │ Directory                │ Files │ Size    │ Cached at│ ││
│      │  ├──────────────────────────┼───────┼─────────┼──────────┼─┤│
│      │  │ /home/u/Pictures/2024    │   312 │  41.2 MB│ 06-07    │🗑││
│      │  │ /home/u/Pictures/holiday │    87 │  11.9 MB│ 06-05    │🗑││
│      │  │ /home/u/Videos           │    14 │   3.1 MB│ 06-01    │🗑││
│      │  │ /home/u/Downloads  ⏳    │    45…│   5.0 MB│ caching… │ ││
│      │  └──────────────────────────┴───────┴─────────┴──────────┴─┘│
│      │                                                              │
│      │  4 directories · 458 files · 61.2 MB total                   │
├──────┴──────────────────────────────────────────────────────────────┤
│ [ image path ]                       [ slider ]  42 files (3 dirs)  │
└──────────────────────────────────────────────────────────────────────┘
```

## Components

### Add-directory form (top)

- A path text input and a **Cache this dir** button.
- Pressing the button validates the path (must exist, must be a
  directory). Invalid paths surface an error toast; nothing is queued.
- A valid path starts the same indexing pipeline the Explorer page
  uses (thumbnails → embeddings), without changing the Explorer's
  selected directory.
- While a run is active, the button is disabled (one run at a time —
  the same single-task constraint that exists today).

### Filter row

- A text input that filters the table to rows whose directory path
  contains the typed substring (case-insensitive). Empty input shows
  all rows.
- A **Refresh** button that reloads the table from the cache database.
  (The table also reloads automatically when a caching run starts or
  finishes.)

### Cache table

One row per **directory** (the parent directory of each cached file;
entries are grouped by their file's immediate parent).

| Column | Content | Notes |
|---|---|---|
| Directory | Absolute canonical path | Truncated head with `…` when too long; full path in tooltip/footer on hover is out of scope for v1 |
| Files | Number of cached files in this directory | Image + video entries combined |
| Size | Sum of the cached files' recorded file sizes | Humanised (KB / MB / GB) |
| Cached at | The **newest** `updated_at` among the directory's entries | Relative or `YYYY-MM-DD HH:MM` local time |
| 🗑 | Per-row clear button | See below |

Rows sort by Cached at, newest first.

#### Per-row clear (🗑)

- Removes every cached entry (image and video namespaces) whose file
  lives directly in that directory, and deletes the associated
  thumbnail files from disk.
- The row disappears from the table on success. Errors surface as a
  toast; the table refreshes regardless so partial deletions are
  visible truthfully.
- No confirmation dialog in v1 — clearing cache is non-destructive to
  the user's media (only recomputation cost). A confirmation step can
  be added later if users ask.
- The clear button is disabled while a caching run is active (writer
  exclusivity).

### In-progress row state

When the user requests caching of a directory:

1. **On start** — a row for the target directory appears immediately
   (or the existing row is marked) with a `⏳ caching…` indicator in
   the Cached at column. Counts/size show current values from the
   database.
2. **On finish** — the table reloads from the database; the indicator
   disappears and the row shows final counts, size, and timestamp.

Runs triggered from the **Explorer** page (directory selection) also
mark the table: if the user navigates to the Cache page during an
Explorer-triggered indexing run, the active directory shows the same
`⏳` indicator.

### Summary footer (inside the page)

Below the table: `N directories · M files · S total`, computed over
the **unfiltered** data set so the user always sees true totals.

## Behaviour details

| Case | Behaviour |
|---|---|
| Empty cache | Table area shows "No cached directories yet." with the add-directory form still usable |
| Filter matches nothing | Table area shows "No match." — summary footer still shows true totals |
| Clear pressed during a run | Button disabled; not possible |
| Cache requested for an already-cached directory | Allowed — the run re-checks freshness; unchanged files are skipped quickly (steady-state no-op) |
| Cache requested for a directory currently shown in Explorer | Allowed — same pipeline, no conflict (single-task rule applies) |
| App restarted mid-run | No persisted run state; the table simply shows whatever the database contains |

## Out of scope (v1)

- Per-file rows or expansion of a directory row into file rows
- Sorting by other columns
- Recursive clear (clearing a directory does not clear subdirectories'
  rows — each row is exactly one directory)
- Progress percentage for the in-flight run (only started/finished
  states are reflected)
- Cache size on disk for thumbnails (Size column counts source file
  sizes recorded in metadata, which is what the database can answer
  cheaply; thumbnail disk usage is roughly uniform per file)
