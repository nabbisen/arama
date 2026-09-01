# RFC 048: Library scale — rendering cost, ordering, and silent truncation

**Status:** Proposed
**Raised by:** the external audit of 0.41.2, 2026-09-01 (findings C2, A6, A8,
C3), verified in
[`external-audit-2026-09-01-verification`](../notes/external-audit-2026-09-01-verification.md)
**Relates to:** [RFC 008](../done/008-gallery-filter-cleanup.md),
[RFC 016](../done/016-cache-capacity.md),
[RFC 036](../done/036-similarity-dialog-absence-states.md)

## Summary

The audit's sharpest sentence is a diagnosis, not a defect:

> arama today is a *folder* tool that describes itself as a *library* tool.

Four findings sit behind it. **All four are defects or quality problems, not
product direction** — that distinction is load-bearing and is the reason this
RFC exists separately from ROADMAP theme C (§5).

## 1. The gallery builds a widget per file, per frame (C2)

`crates/ui/main/src/core/views/gallery/view.rs` iterates the complete file map
and constructs an `Element` for every entry on every redraw. There is no
windowing and no recycling.

The consequence is not a slow gallery — it is a **ceiling on library size**, and
the ceiling is already visible in the product's own surfaces: `faq.md` documents
the symptom and offers "reduce the scope" as the remedy, and the subdirectory
depth setting caps at 2. **A performance constraint has been turned into a
setting and then into advice.**

That is the finding worth acting on. The audit's comparison is fair: digiKam's
similarity engine assumes 100 k+ item libraries; arama's rendering model assumes
a folder.

## 2. Files render in hash order (A6)

`gallery.rs:15`:

```rust
dir_path_thumbnail_path_map: BTreeMap<PathBuf, FastHashMap<String, String>>
```

The **outer** map — directories — is already ordered. The **inner** map — the
files inside each directory — is a `FastHashMap`, so files appear in hash order,
which is stable within a run and arbitrary between them.

There is no sort control anywhere in the UI, so this is not "the default order
is unusual" — it is **the only order, and it is meaningless**. A file manager
that cannot list files in a predictable order is missing something more basic
than a feature.

The fix is one type. Anything beyond it — user-selectable sort by name, date,
size — is product direction and out of scope here (§5).

## 3. Similar pairs truncate silently, and asymmetrically (A8)

`similar_pairs_dialog.rs:33`:

```rust
const MAX_IMAGE_SIMILAR_PAIRS: usize = 50;
```

Images are capped at 50. **Videos have no cap at all.** Nobody chose that
asymmetry; it is what happens when a bound is added on one path.

Two problems, and the second is the one that matters:

1. **The cap is invisible.** A user with 300 similar pairs sees 50 and is told
   nothing. There is no "showing 50 of 300", no indication that a threshold
   exists.
2. **It is indistinguishable from completeness.** RFC 036 built absence states
   precisely so "nothing found" could never be confused with "something went
   wrong". A silent truncation reintroduces the same class of ambiguity one
   level up: *"these are the similar pairs"* and *"these are the first 50 of the
   similar pairs"* render identically.

**A single explicit cap applied to both media types, with retained-versus-total
counts rendered**, replaces a silent truncation with an honest one. That is the
whole change; raising or removing the cap is a separate question that depends on
§1.

## 4. `view()` does filesystem I/O, per frame (C3)

Verified at all four named sites:

| Site | Per-frame syscall |
|---|---|
| `views/setup/view.rs:78` | `DiskSpace::new` — `statvfs` / `GetDiskFreeSpaceEx` |
| `components/setup/downloader/view.rs:131` | `DiskSpace::new` |
| `settings_dialog/tab/file_system_settings/view.rs:17` | `DiskSpace::new` |
| `layout/src/footer/view.rs:24` | `canonicalize()` on the hovered path |

Two of the disk-space calls are on the **setup screen, which shows an animated
download progress bar** — so it redraws continuously while making a filesystem
syscall each frame. The footer canonicalises on every frame of mouse movement
across the gallery.

`view()` should be a pure function of state. Each of these hoists into state
refreshed by the message that can change it. This is small, and it is listed
here rather than as a stray task because it shares a cause with §1: **cost that
scales with redraws rather than with events.**

## 5. What this RFC deliberately does not do

**It does not populate ROADMAP theme C.** The audit's long-term list proposes
multi-select and batch actions in the pairs dialog, a perceptual-hash /
exact-duplicate mode, and user-selectable sort. Those are product direction —
statements about what an arama user should be able to *do* — and theme C's own
text reserves that to the owner:

> The architect still does not populate this section. […] it stays open until
> the owner states what an arama user should be able to *do* that they currently
> cannot.

That rule was written for exactly this situation: a plausible, well-argued
external list of features arriving with no owner behind it. **The candidates are
recorded in the audit and in the schedule as offered, not adopted.** The owner
may take any, all, or none of them.

The line this RFC draws: **§1–§4 are things arama does incorrectly or
unpredictably today.** Sorting arbitrarily is a defect. Offering a sort menu is a
feature. Truncating silently is a defect. Multi-select is a feature.

## 6. Non-goals

- Multi-select, batch delete, sort controls, duplicate-detection modes — §5.
- Replacing exact O(N²) search with an ANN index.
  [RFC 047](./047-indexing-cancellation.md) §1.3 argues exact search is right at
  this scale; it needs a bound, not an index.
- Raising the depth-2 subdirectory cap. That becomes a real question once §1
  lands, and answering it before then would be guessing.
- Thumbnail storage or cache capacity — [RFC 016](../done/016-cache-capacity.md).

## 7. Acceptance

- The gallery's cost is bounded by what is visible, not by directory size,
  demonstrated with a measurement at two library sizes rather than an assertion.
- Files render in a stable, predictable order across runs.
- One cap, both media types, with retained-versus-total counts visible.
- No `view()` performs filesystem I/O.

## 8. Open question

**What library size should arama target?** §1's fix removes a ceiling without
setting one, and every remaining answer — the depth cap, the pair cap, whether
exact search still suits — depends on the number. It has never been stated.
Unlike §5's items this is not product direction but a sizing constraint, and the
architect can propose it once §1 makes the current ceiling measurable.
