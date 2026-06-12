# Using arama

## The Explorer page

The Explorer page is the default view after setup. It has three
areas:

```
┌──────────────────────────────────────────────────────────┐
│  📁  │  [ directory path input ]       [ ⊞ pairs ]      │
│  🗃  ├────────────────┬─────────────────────────────────┤
│  ⚙   │  Directory     │  Gallery                        │
│      │  tree          │  ┌──┐ ┌──┐ ┌──┐                │
│      │                │  │  │ │  │ │  │                │
│      │                │  └──┘ └──┘ └──┘                │
│      │                │                                 │
├──────┴────────────────┴─────────────────────────────────┤
│  [ image path ]          [ slider ]   42 files (3 dirs) │
└──────────────────────────────────────────────────────────┘
```

**Side nav** (far left) — three icon buttons:
- 📁 Explorer (current page)
- 🗃 Cache
- ⚙ Settings

**Header bar** — directory path input and the **Similarity Pairs**
button (the pairs button is enabled only after indexing completes).

**Directory tree** (left panel) — shows the folder hierarchy. Click
any folder to select it and start indexing its contents.

**Gallery** (right panel) — displays thumbnails for all indexed files
in the selected directory. Thumbnail size is controlled by the slider
in the footer.

**Footer** — shows the hovered file path, a thumbnail-size slider
(128 – 384 px), and file/directory counts.

## Selecting a directory

Click any folder in the directory tree. arama immediately:

1. Loads the directory structure.
2. Starts a background indexing pass: generates thumbnails for any
   file not yet cached, then computes CLIP (and wav2vec2 for video)
   embeddings for files that don't have them yet.

The spinning indicators on folder icons in the tree show that indexing
is in progress. The Similarity Pairs button is greyed out until
indexing finishes.

**Switching directories mid-index** is safe: the running indexing task
is cancelled automatically and a fresh one starts for the new selection.

## Browsing the gallery

The gallery shows thumbnails for every file in the selected directory
(subdirectory depth is controlled in Settings). Hover over a thumbnail
to see its full path in the footer.

Use the **thumbnail size slider** in the footer to zoom in or out
(range: 128 – 384 px).

Type in the **filter box** at the top of the gallery to narrow the
display to files whose filename contains the typed text
(case-insensitive). A `N of M` count shows how many files match. Press
✕ or clear the box to show all files again. The filter resets when you
select a different directory.

Right-clicking a thumbnail opens a **context menu** with:
- Open file
- Show in folder
- Move to trash

## Finding similar media (focus view)

Click any thumbnail to open the **focus view** for that file. The focus
view shows files sorted by similarity to the selected item, highest
first, filtered to those above the similarity threshold (0.86 by
default).

The scope of the search is set using the **Cache lookup strategy**
pick list in the focus view:
- **Everywhere** — all indexed files across the directory tree
- **Current directory and subdirectories** — files under the same
  parent folder and below
- **Current directory only** — files in the same immediate folder

Navigate the focus view history with the back/forward controls.

## Similarity Pairs

Click the **⊞** button in the header to open the **Similarity Pairs**
dialog. This scans every pair of indexed files and surfaces those
whose similarity exceeds the threshold, grouped by score. Use this
to find near-duplicate images or videos across a large library.

The scan runs asynchronously; results appear progressively.
