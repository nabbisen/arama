# The footer is untranslated, and its directory plural counts files

**Found:** 2026-08-20, in the footer of a Japanese-locale screenshot taken for
RFC 043. Not looked for.
**Status:** recorded, not scheduled. Both are small, both are in the same two
files, and they would make one task.

## 1. Four hardcoded English strings

```text
crates/ui/layout/src/footer/thumbnail_size_slider/view.rs:16   text("Thumbnail size")
crates/ui/layout/src/footer/view.rs:12,14                      "files" / "file"
crates/ui/layout/src/footer/view.rs:16                         "dirs" / "dir"
crates/ui/layout/src/footer/view.rs:39                         "({} {} scanned)"
```

A Japanese user sees **`Thumbnail size`** and **`0 file (1 dir scanned)`** in
English, on every screen, permanently. arama ships two locales and this is the
one surface that is always visible.

Every other user-facing string in `app/src` and `crates/ui` goes through
`t(...)`. These four are the exceptions, and nothing enforces the rule.

## 2. The directory plural is decided by the file count

`crates/ui/layout/src/footer/view.rs:16`:

```rust
let dirs_label = if 1 < self.files_count { "dirs" } else { "dir" };
```

`files_count`, not `dirs_count`. So:

- 27 files in one directory → **"27 files (1 dirs scanned)"**
- 1 file across three directories → **"1 file (3 dir scanned)"**

Cosmetic, and wrong in both directions.

## 3. How long it has been visible

**"27 files (1 dirs scanned)" is legible in review packages 117, 119 and 121**,
across five days of capture review by both the dev team and the architect. Every
one of those reviews sampled pixels, computed contrast ratios, and compared
md5s — and none of us read the sentence in the corner of the screen.

That is the finding worth keeping. The captures were being examined as
*evidence for a specific claim*, and everything outside the claim was invisible.
A reviewer looking for a border does not read the footer.

**No process change is proposed here.** "Look at the whole screenshot" is not a
control, and pretending otherwise would be worse than recording the miss.

## 4. If this is fixed

The plural fix is one identifier. The i18n fix needs four keys in both `en.rs`
and `ja.rs`, and the count strings are formatted with interpolation — Japanese
does not pluralise, so `t()` keys should carry the whole phrase rather than a
noun the caller pluralises.

**Do not add a generic pluralisation helper for this.** Two locales, one of
which has no plural forms, does not justify the machinery.
