# snora 0.40 → 0.42 — assessment against arama

snora sent a combined letter on 2026-09-02 covering 0.41.0, 0.41.1 and 0.42.0,
after the 0.40.0 note of 2026-08-25. arama is on **0.39.1**.

**Outcome: upgrade, and do it before Task 039.** The reasoning below is what
changed my earlier recommendation, which was to leave snora alone until a
release carried content for us. One does.

## 1. The overlay pointer bug reaches arama, and it reaches Task 039

snora 0.41.0 fixed overlays not containing pointer events: *"Four surfaces
should have contained pointer input; only the sheet did."*

**Verified in the version arama actually ships**, not taken from the letter.
In `snora-0.39.1`:

- `src/overlay/sheet.rs:41` carries the comment *"Sheet content must capture its
  own events; otherwise clicks inside…"* — the sheet captures.
- `src/overlay/dialog.rs` contains **no** `mouse_area`, no capture, no event
  handling of any kind.

arama uses `snora::Dialog` for the MediaFocus and SimilarPairs modals
(`app/src/core/view.rs:184`) under an `AppLayout` that sets
`.on_close_modals(Message::DialogClose)` (`:169`).

**Expected consequence, stated as expected rather than observed:** a click on a
dialog's own padding or plain text is not captured by the dialog, so it reaches
the layout's close handler and dismisses the modal. snora lists exactly this as
the first symptom. **This has not been reproduced by observation here** — the
mechanism is verified, the behaviour is inferred, and RFC 040's rendered-evidence
method is how it should be confirmed. Asserting modal behaviour from reading
alone is a mistake this project has already made once (Task 028).

**Why it gates Task 039.** Task 039 adds arama's first confirm-before-delete
dialog, over a destructive action that wipes the cache. snora's own letter names
this case:

> *"A modal that does not block input is a UI-integrity problem — a dialog could
> be bypassed by clicking through it to the control it was meant to be
> guarding."*

Self-dismissal fails safe (it cancels). **Click-through does not** — it can reach
the Delete control the dialog exists to guard. Building the confirmation on
0.39.1 would ship a guard with a hole in it.

## 2. What each item costs arama

| Item | Release | Applies? |
|---|---|---|
| Overlay pointer containment | 0.41.0 | **Yes** — §1. The reason to upgrade. |
| Widget-layer colour pairing repair | 0.41.0 | Yes — arama has the `design` feature. |
| Toast `Warning` text white → black | 0.42.0 | **Yes** — 3.18:1 → 6.60:1, below the 4.5:1 AA floor today. arama renders toasts through `AppLayout`. |
| Toast `Info` fill and text | 0.42.0 | Yes — 4.43:1 → 5.63:1. |
| Dismiss `×` no longer fades | 0.42.0 | Yes — worst case 3.38:1 → 4.83:1. |
| Visual baselines with a toast invalidated | 0.42.0 | Yes — costs a recapture pass. |
| `iced` `canvas`/`svg` no longer transitive | 0.42.0 | **No** — verified: arama uses neither `iced::widget::canvas` nor `::svg` anywhere. |
| `lucide-icons` no longer enables `advanced` | 0.40.0 | **No** — verified 2026-08-25: zero `iced::advanced::` uses. |

**Both breaking changes miss arama.** The two `Warning`/`Info` contrast repairs
are live AA failures in arama's own UI today, which is a second reason to go to
0.42.0 rather than stopping at 0.41.x.

## 3. The withdrawn WCAG 1.4.1 claim — no action, and the reason matters

snora withdrew: *"Toast intents and notice tones are distinguishable by more than
colour alone in snora's prefab widgets."* Their prefabs vary only background and
accent colour; identical title and body, no icon, no prefix.

**arama does not inherit the defect, because arama supplies its own text.**
Every toast is constructed with a per-site title and body
(`app/src/core.rs:353-373` — `push_error_toast`, `push_success_toast`,
`push_warning_toast`, each taking `title` and `body`), so intent is carried by
words as well as colour.

**And arama makes no 1.4.1 claim to withdraw.** Searched `docs/src`, `README.md`
and the whole RFC corpus: the WCAG references are to AA *contrast* (RFC 010,
RFC 011, `workspace.md:133`) and SC 2.1.1 *Keyboard* (RFC 044). None cites
snora's Use-of-Colour claim.

**Worth confirming during the upgrade rather than assuming:** that each toast
title actually names its situation in both locales. "Distinguishable by text"
is only true if the text distinguishes.

## 4. Sequencing

1. **snora 0.39.1 → 0.42.0**, one hop. Two upgrade passes cost two
   rendered-evidence rounds; the intermediate versions carry nothing arama needs
   on its own.
2. **Then Task 039**, whose confirmation dialog is sound only on 0.41.0+.
3. Task 042 and the rest of the audit slate are unaffected and need not wait —
   042 touches only `crates/ai/src/pipeline/encode/audio/`.

**The upgrade needs its own captures** and must not be bundled into a
visual-change diff — RFC 040 §3.1 and the RFC 043 handoff's sequencing amendment
both exist because a bundled upgrade makes every rendered difference
unattributable. The toast appearance changes in 0.42.0 make that sharper here
than usual.

## Related

`rfcs/done/040-snora-0.29-upgrade-and-dialog-surface.md` ·
`rfcs/notes/dialog-card-edge-contrast-measured.md` ·
`.git-exclude/tasks/dev-team/039-cache-delete-confirmation.md`
