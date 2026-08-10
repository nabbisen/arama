# RFC 036: Similarity-dialog absence states

**Status.** Implemented (0.39.0). Accepted by the project owner 2026-08-10;
design questions 1–3 were settled in the handoff, and question 1's answer
**corrects this RFC** — see below.

*As built:* both dialogs render from one shared mechanism in
`similarity_read_outcome.rs`. One interaction the handoff did not specify was
found and handled during implementation: when a read failure is present the
absence message is **suppressed**, because "results may be incomplete" and a
confident "nothing similar found" directly beneath it would contradict each
other.

*Known limitation, deliberately out of scope:* the dialog text is legible only
where it happens to fall on neutral background. `snora`'s dialog overlay draws
no card, so a message can land over gallery thumbnails. RFC 036's binding rule
— no dialog renders zero text — is met; making that text reliably readable is
upstream work in snora, evidenced by this RFC's own rendered captures.
**Tracks.** Two gaps found during RFC 035's implementation and review, both
deliberately left out of that RFC's scope: the similarity dialogs are silent in
situations where nothing went wrong but the user is still owed an explanation.
**Touches.** `crates/ui/widgets/src/dialog/similar_pairs_dialog/view.rs`,
`crates/ui/widgets/src/dialog/media_focus_dialog/{view.rs,similar_media.rs}`,
`crates/i18n`. Possibly `snora`'s dialog overlay — see Design question 1.

## Summary

RFC 035 made the similarity dialogs tell the user when a cache read **failed**.
It deliberately did not address the cases where a read **succeeded** and there
is simply nothing to show. Two of those remain, and in both the dialog says
nothing at all.

They are separable from RFC 035 and from each other, but they share a shape —
*absence rendered as silence* — and fixing them independently would produce two
mechanisms for one problem, which is the reasoning RFC 035 itself applied.

## Why now

Both were found with evidence during RFC 035's cycle, not speculated:

- **Gap 1** was captured in a rendered screenshot during RFC 035's own smoke
  run: the Similar Pairs dialog, opened against a healthy cache yielding zero
  pairs, showing the gallery dimmed by the overlay backdrop and **no dialog
  text of any kind** — no title, no message, no card. The capture lives with
  the project's internal review records, which are not distributed in the
  source archive; the observation is restated in full under Gap 1 below so this
  RFC stands on its own.
- **Gap 2** was identified while scoping RFC 035 §3.1 and recorded there as a
  knowing exclusion.

Neither is a regression from RFC 035. Both predate it. But RFC 035 is what made
them visible: once failure has a voice, the remaining silences stand out.

## Gap 1 — an empty similar-pairs dialog renders nothing at all

**Observed.** Open Similar Pairs against a healthy cache that yields zero
pairs. The dialog shows no title, no message, no card — only a dimmed backdrop
over the gallery. It is visually indistinguishable from a dialog still loading,
or from a misfire.

**Mechanism.** `similar_pairs_dialog/update.rs` sets `self.pairs =
Some(outcome.items)` unconditionally, including when `items` is empty.
`view.rs`'s only "nothing here" message, `t("pairs.no_valid")`, is gated on
`self.pairs == None`, which holds for at most one frame after the dialog opens.
A legitimate empty result therefore falls through to the populated-results
branch with zero rows. `snora`'s dialog overlay draws only a dim backdrop, no
card or panel, so zero rows means zero pixels.

**Why it matters more than it sounds.** This is the state a first-run user
reaches most easily — media present, indexing not yet done, so no embeddings and
no pairs. The application appears broken at exactly the moment a new user is
deciding whether it works. It is the same class as the false first-run error
fixed during 0.37.0, inverted: that one said something wrong, this one says
nothing at all.

**`SMOKE-SIMILARITY-SPARSE` passes over it.** Its wording — "degrades to partial
or empty results instead of crashing" — is satisfied by a dialog that renders
nothing, because not crashing is all it asserts. The row needs sharpening
whatever else this RFC decides.

## Gap 2 — video results absent because ffmpeg is missing

**Observed.** A user with indexed videos and no `ffmpeg`/`ffprobe` pair gets an
image-only result set, with nothing in the dialog explaining why videos are
absent.

**Why RFC 035 excluded it, correctly.** Nothing failed to be read — video
comparison never ran. Folding it into "some files could not be read" would
assert a mechanism that did not occur. And ffmpeg absence already has a
dedicated, actionable surface: the `settings.ai.ffmpeg_*` states, including one
naming exactly what to install.

**Why it is still a gap.** "There is a place elsewhere in the app that would
explain this if you went looking" is not the same as telling the user. The
result set is smaller than the truth and nothing on screen says so.

**Why it is genuinely harder than Gap 1.** Surfacing it means rendering a
*configuration state* inside a results dialog. That is a different kind of
message from both "this failed" and "there is nothing here", and it invites a
general question this project has not answered: which configuration states are
allowed to speak from which surfaces. Getting that wrong produces nagging.

## Goals

- A user who opens a similarity dialog always learns which of these is true:
  results are shown, there are none, something failed, or something is not
  configured.
- One mechanism, shared by both dialogs, consistent with RFC 035's inline
  message rather than parallel to it.
- No new error-UX vocabulary. Classify within RFC 017's existing tiers.

## Non-goals

- Similarity scoring, thresholds, or ranking.
- Any change to RFC 035's failure routing, which is correct as shipped.
- Prompting, nagging, or calls to action inside the results dialog. Settings
  remains where configuration is *changed*; this is only about whether the
  dialog admits what it is missing.
- A general audit of every empty state in the application. If that is wanted it
  is its own RFC; this one covers the two dialogs RFC 035 touched.

## Design questions this RFC must settle

### 1. Does the dialog need a frame at all?

Gap 1's root cause is partly that `snora`'s overlay draws no card, so a dialog
with no content is invisible. Two directions:

- **Fix in arama** — guarantee the dialog always renders at least a title and a
  status line, so content emptiness never means pixel emptiness.
- **Fix in snora** — give the dialog overlay a card/background, so any dialog is
  visibly present regardless of content.

The second is the deeper fix and affects every dialog, not just these two. It is
also a first-party dependency change with its own release cycle. **This RFC
should decide the direction before any implementation.**

> **Settled 2026-08-10 — and this framing was wrong.** I wrote that "the first
> approach becomes redundant if the second lands." It does not. Reading the
> source settles both halves:
>
> `snora-0.25.0/src/overlay/dialog.rs` is fifteen lines and its whole body is
> `center(dialog.content)`. There is no container, background, or padding — and
> its own doc comment calls it *"the centered modal card"*, so snora does not
> draw what it documents.
>
> But a card with no text inside it still tells the user nothing. "No similar
> items found" is information they need whether or not a frame surrounds it.
> **The two fixes are orthogonal, not alternatives.**
>
> **Direction: fix in arama.** Guarantee the dialog always renders text, so
> content-emptiness can never mean pixel-emptiness. The snora card is a real
> improvement that benefits every dialog in every consumer, and should be
> raised upstream as its own item — but it does not block this work and this
> work does not wait for it.

### 2. Is "nothing found" the same message as "not indexed yet"?

A zero-pair result has at least two causes a user would act on differently:
nothing similar exists, or nothing has been indexed. The data to distinguish
them is available — RFC 035's plumbing already separates an unindexed target
from an empty cache. Whether to distinguish them in the UI is a UX decision.

Recommendation: **distinguish them.** "No similar items found" and "Nothing has
been indexed yet" send the user to different next actions, and conflating them
recreates the ambiguity this RFC exists to remove.

### 3. How does Gap 2's message coexist with RFC 035's?

If a read failure and a missing toolchain occur together, does the dialog show
one message or two? RFC 035 settled "one aggregated message per dialog open"
for failures. Extending that literally would merge two unrelated statements.

Recommendation: **one inline region, at most two sentences** — one about
failures, one about unavailable capability — never repeated per item. This
preserves the intent of RFC 035's decision 3 (no per-item spam) without
pretending two different conditions are one.

## Testing and verification

- An empty-but-successful result renders visible, readable text in both dialogs.
- An unindexed target is distinguishable from a genuinely empty result, if
  Design question 2 resolves that way.
- Missing ffmpeg with indexed videos produces exactly one statement about it.
- A failure and a missing toolchain together produce a bounded message, not two
  stacked mechanisms.
- Both locales resolve every new key, asserted by test — RFC 032's cycle showed
  a missing `ja` entry renders a raw key.
- **Rendered evidence for every state above**, using the method established in
  RFC 035's package: scratch-profile isolation under `.git-exclude/tmp/`, native
  Wayland window capture. Gap 1 is invisible to unit tests by construction — it
  is a rendering outcome, and only a screenshot can show it is fixed.
- Sharpen `SMOKE-SIMILARITY-SPARSE` so that "renders nothing" fails it.

## Acceptance criteria

- No similarity dialog can present a state in which it renders no text.
- The four outcomes in Goals are each distinguishable on screen.
- One shared mechanism across both dialogs, extending RFC 035's inline region.
- `SMOKE-SIMILARITY-SPARSE` sharpened; new smoke rows added **and executed**.
- Both locales verified by test.

## Risks

- **Nagging.** Gap 2's message could become a permanent scold for users who
  deliberately do not want video support. Mitigation: state the fact once,
  inline, with no call to action; do not repeat per item.
- **Scope drift into a general empty-state audit.** The Non-goals fence it. If
  the audit is wanted, it is a separate RFC.
- **Deciding question 1 the shallow way.** Guaranteeing text in these two
  dialogs fixes the symptom; an overlay that never draws a frame will keep
  producing this defect elsewhere. The cheap fix should be chosen knowingly, not
  by default.

## Open questions

- Does `snora`'s dialog overlay change (question 1) belong to arama's roadmap at
  all, or is it upstream work with its own timing?
- Should this land before or after 0.38.0? Neither gap is a regression, so
  deferring is defensible; both are small and visible, so including them is too.
  **Owner's call.**
