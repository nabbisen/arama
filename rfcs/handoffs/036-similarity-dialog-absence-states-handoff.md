# RFC 036 Handoff — Similarity-dialog absence states

Companion to [RFC 036](../done/036-similarity-dialog-absence-states.md),
which shipped in **0.39.0** and moved to `rfcs/done/` with that cut, per
[RFC 000](../done/000-rfc-lifecycle-policy.md).

## 1. Design authority

1. [RFC 036](../done/036-similarity-dialog-absence-states.md) — the
   governing design;
2. [RFC 035](../done/035-similarity-dialog-error-routing.md) — the failure
   routing this extends. **Do not change it**; it is correct as shipped;
3. [RFC 017](../done/017-visible-recoverable-error-ux.md) — the tier model.

## 2. The three design questions, settled

### 2.1 Fix in arama, not in snora

`snora-0.25.0/src/overlay/dialog.rs` is fifteen lines whose entire body is
`center(dialog.content)` — no container, no background, no padding — while its
doc comment calls it *"the centered modal card"*.

RFC 036 originally framed the arama-side fix as redundant if snora gained a
card. **That was wrong, and the RFC now records the correction.** A card with no
text inside still tells the user nothing. The two are orthogonal.

**Implement the arama-side fix. Do not modify snora as part of this work**, and
do not wait for it. Raising the missing card with snora is worthwhile and
separate.

### 2.2 Distinguish "nothing found" from "not indexed"

**Yes, distinguish them.** They lead to different user actions, and RFC 035's
plumbing already separates an unindexed target from an empty cache — the data is
in hand. Conflating them recreates the ambiguity this RFC exists to remove.

### 2.3 One inline region, at most two sentences

Gap 2's message shares RFC 035's inline region rather than getting a mechanism of
its own — but as a **separate sentence**, not merged into the failure text.
Never repeated per item.

RFC 035 decision 3 ("one aggregated message per dialog open") forbids per-item
spam; it does not require pretending two different conditions are one.

## 3. Required implementation

The dialog must always tell the user which of these is true:

| State | Message |
|---|---|
| Results present | the results |
| Read failure | RFC 035's existing message — **unchanged** |
| Nothing indexed yet | a distinct message |
| Nothing similar found | a distinct message |
| ffmpeg missing, videos indexed | a statement that video comparison did not run |

**The binding rule: no similarity dialog may render zero text.** That is the
acceptance criterion Gap 1 is really about. A dialog that finds nothing must be
as legible as one that finds something.

Both dialogs — `similar_pairs_dialog` and `media_focus_dialog` — behave
identically, sharing one mechanism. They diverged once before; RFC 035 fixed
that by construction with a shared type, and that pattern should be extended
rather than duplicated.

### The Gap 1 mechanism, precisely

`similar_pairs_dialog/update.rs` sets `pairs = Some(items)` unconditionally, so
`view.rs`'s `pairs == None` gate for `t("pairs.no_valid")` is true for at most
one frame. A legitimate empty result falls through to the populated branch with
zero rows, and snora draws no card, so zero rows means zero pixels.

Fixing the gate alone is not sufficient — see §4's ffmpeg case, which also
produces a result set that needs explaining.

## 4. Gap 2 — the message must not misdescribe

Missing ffmpeg means video comparison **never ran**. It is not a read failure and
must not be reported as one — RFC 035 §3.1 excluded it for exactly that reason,
and that exclusion stands.

State the fact, once, inline: video results are unavailable because no
ffmpeg/ffprobe pair was found. **No call to action, no link, no prompt.**
Settings → AI already carries the actionable `settings.ai.ffmpeg_*` states
including what to install; this dialog's job is only to stop lying by omission.

The failure mode to avoid is nagging a user who has deliberately chosen not to
install ffmpeg.

## 5. Change scope

- `crates/ui/widgets/src/dialog/similar_pairs_dialog/{view.rs,update.rs}` and
  siblings;
- `crates/ui/widgets/src/dialog/media_focus_dialog/{view.rs,update.rs,similar_media.rs}`;
- `crates/ui/widgets/src/dialog/similarity_read_outcome.rs` if the shared type
  is the right place to carry the extra state — prefer extending it over adding
  a parallel channel;
- `crates/i18n/src/{en,ja}.rs` — new strings;
- `docs/src/dev/testing.md` and the smoke evidence template.

## 6. Non-change scope

- RFC 035's failure routing and its message. Correct as shipped.
- `snora`. See §2.1.
- Similarity scoring, thresholds, ranking.
- `arama-cache` or its error types.
- A general empty-state audit of the whole application. If wanted, it is its own
  RFC.
- Any new error-UX mechanism. Use what RFC 017 provides.

## 7. Required tests and evidence

- Each of §3's five states renders **visible, non-empty text**, in both dialogs.
- An unindexed target is distinguishable from a genuinely empty result.
- Missing ffmpeg with indexed videos produces exactly one statement about it.
- A read failure **and** missing ffmpeg together produce a bounded message — two
  sentences at most, not two stacked mechanisms.
- RFC 035's existing assertions still pass unchanged. Its §3.1 exclusions —
  unindexed target and missing ffmpeg produce no *error* — must not regress;
  they now produce a non-error message instead, which is a different assertion.
- Both locales resolve every new key, asserted by test.

**Rendered evidence is mandatory and is the only proof that matters for Gap 1.**
It is a rendering outcome: a unit test cannot show that a dialog draws no
pixels. Use RFC 035's method — scratch-profile isolation under
`.git-exclude/tmp/`, never the owner's real profile, native Wayland
window-scoped capture.

Capture at minimum: an empty-but-successful result, an unindexed target, and the
ffmpeg case.

**Sharpen `SMOKE-SIMILARITY-SPARSE`.** Its current wording — "degrades to
partial or empty results instead of crashing" — is satisfied by a dialog that
renders nothing, which is how Gap 1 survived. Rewrite it so rendering nothing
fails it. Add new smoke rows as needed and **execute them**.

## 8. Acceptance criteria

- No similarity dialog can render zero text in any state.
- The five states in §3 are each distinguishable on screen.
- One shared mechanism across both dialogs; behaviour identical.
- RFC 035's routing and tests unchanged and passing.
- Both locales verified by test.
- `SMOKE-SIMILARITY-SPARSE` sharpened; new rows added **and executed**.
- Rendered evidence for the three states named in §7.

## 9. Known risks

- **Nagging.** Gap 2's message becoming a permanent scold. §4 is the guard.
- **Over-surfacing.** Turning an ordinary empty state into something that reads
  like an error is a worse defect than the silence being fixed.
- **Divergence between the dialogs.** Implement once, share; do not fix each
  separately.

## 10. Required deliverables

A review request under `.git-exclude/review-request/NNN-<slug>/` opening with an
`## Entry point` section giving the pinned commit, the exact `git show` command,
and plain paths to every file. Include the rendered evidence and the smoke
results. Report observed output; a check not run is recorded as not run.
