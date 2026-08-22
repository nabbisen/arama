# Invariant plural counts is a chosen convention, not an inherited default

**Found:** 2026-08-22, Task 034 §3.1 (source: package 124 §4's sweep, itself
a consequence of Task 031's footer fix).
**Status:** decided — accepted, recorded here so it stops being
rediscovered.

## 1. The pattern

arama renders `1 files`, `1 directories`, `1 dirs scanned` — the English
noun never varies with the count. At least three places do this today:
the footer (`files_line`/`dirs_line`, `crates/ui/layout/src/footer/view.rs`,
Task 031), the cache page summary (`cache.summary.files`/`directories`/
`total`, `crates/ui/main/src/core/views/cache_page/view.rs`), and this
task's own new keys (`cache.summary_report.files_skipped`/
`cache_writes_failed`/`files_indexed`, `crates/i18n/src/en.rs`).

Japanese is correct in every one of these — it has no plural forms, so
the same noun is exactly right regardless of count. English is not: "1
files" reads wrong to a fluent reader, in a way "1 dirs scanned" and "1
directories" do too.

## 2. The premise this task started from was wrong, and worth correcting

Task 034 framed fixing this as needing "a mechanism `arama-i18n` does not
currently have… a real design change and probably its own RFC."

**That premise is false.** `arama-i18n` already has and uses exactly the
mechanism this would need: `crates/ui/main/src/core/views/cache_page/view.rs`'s
`format_relative_timestamp_at` selects between `cache.time.minute` /
`cache.time.minutes` (and the hour/day/month/year pairs alongside them,
`crates/i18n/src/en.rs:105-114`) with a plain `if value == 1 { t(singular_key)
} else { t(plural_key) }`. Two keys per concept, one runtime branch, zero
new code in `arama-i18n` itself. This has shipped correctly since before
this task started.

So applying the same two-key pattern to the file/directory counts would
cost: 2×N new key pairs (N = number of count sites) and one `if value ==
1` branch per call site. Not an RFC-sized change — a same-shape edit to
sites this task and Task 031 already touched.

## 3. Decided anyway: accept the invariant form, for now

**Accepted as-is**, not fixed in this task or Task 031. Reasons, in order
of weight:

1. **Task 031 already made this call and the architect endorsed it**,
   independently of this (mistaken) "needs an RFC" framing: "arama
   already renders `1 directories` · `1 files` on the cache page" was
   cited as existing, accepted precedent when reviewing package 124.
   Reopening it here, in a task about untranslated strings, would relitigate
   a decision one task ago for reasons that turned out not to hold.
2. **The actual defect this task and Task 031 both exist to fix — English
   text with no translation at all — is orthogonal to this.** Every
   string this task added is translated and grammatically correct in
   Japanese; only English's singular form is imperfect. Different defect
   class, does not block either task's own acceptance criteria.
3. **Scope discipline.** Retrofitting the existing sites (footer: 2 keys →
   4; cache page summary: 3 keys → 6; this task's own 3 report keys → 6)
   is real, contained work — but it is *additional* work invented mid-task
   by a corrected premise, not what either task was asked to do.

## 4. What would change this

If the owner wants English pluralisation fixed, the cost is now known
precisely (§2) and the pattern to copy is `cache.time.*` plus
`format_relative_timestamp_at`'s `if value == 1` branch — not a new RFC,
not new `arama-i18n` machinery. A small, contained follow-up task, the
same shape as this one, would do it. Until then: **invariant plural
nouns for counts is arama's convention, chosen, not inherited** — the
next sweep should read this note rather than raise the question again.
