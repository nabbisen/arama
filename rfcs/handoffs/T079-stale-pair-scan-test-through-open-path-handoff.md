> **ACCEPTED 2026-11-17 — review 182.**

> **AUTHORIZED 2026-11-17 — owner:** *"The three are approved."* (T077, T078,
> T079). Written 2026-11-17 by the architect.
>
> **Independent of every open task.** Follow-up 1 of review 170 (T061).

# T079 Handoff — prove the stale pair-scan guard through the real open path

## 1. Purpose

T061's second rework (review 170) made each Similar Pairs scan carry a
generation, so that a cancelled scan's late result is dropped instead of being
shown as the new scan's. Its test,
`stale_embeddings_ready_from_a_closed_scan_is_dropped_the_current_one_is_accepted`
(`app/src/core/tests.rs`), **sets `similar_pairs_scan_generation = 2` by hand**.
It never runs the open handler. So deleting the `+= 1` in that handler — the line
the whole guard depends on — would still pass. The architect specified that test,
and it met the spec; the spec was too weak. This task closes the gap.

## 2. Background

- The open handler: `header::message::Event::SimilarPairsDialogOpen` in
  `app/src/core/update/component.rs`. It needs `self.dir_node` to be `Some`,
  increments `similar_pairs_scan_generation`, calls
  `dialog.default_task(generation)`, and stores the scan's `ScanControl`.
- **No scan runs in a test.** `App::update` returns the scan as an iced `Task`,
  which only the runtime executes. Calling the open event through `App::update`
  increments and stores, and starts nothing. This is why the test can use the
  real path.
- Closing goes through the dialog-close message, which cancels the stored control
  (`handle_dialog_close`).

## 3. Applicable RFC and requirements

[RFC 047](../done/047-indexing-cancellation.md) §3.3; RFC 036 (no result that
looks complete and is not).

## 4. Change scope

`app/src/core/tests.rs` only.

## 5. Explicit non-change scope

All production code. If the test cannot be written without a production change,
stop and report what is missing.

## 6. Required implementation

A test, through `App::update` only, after `App::new()` under the usual
`ARAMA_DATA_HOME_TEST_LOCK` isolation:

1. Give the app a `dir_node` (the same fixture shape the existing test uses).
2. Send the **real open event**: generation 1 is assigned, and a control is stored.
3. Send the **real close message**: the control is cancelled.
4. Send the **real open event** again: generation 2, a new control.
5. Deliver `EmbeddingsReady(1, …)`: **dropped** — the dialog is still scanning,
   and the stored control is still scan 2's (`Arc::ptr_eq`).
6. Deliver `EmbeddingsReady(2, …)`: **accepted** — not scanning; the control is
   cleared.

Read the generations from where the handler stores them; do not set them.

**Show that it can fail:** in a scratch worktree with its own target directory,
remove the `+= 1` from the open handler. The new test fails at step 5; the old
test still passes. Report both outputs, and leave nothing of it in the commit.

Keep the existing test; it documents the delivery check in isolation.

## 7. Required tests

The new test, five runs locally, and all gates: `fmt --check`, `clippy
--all-targets -D warnings`, `test --workspace`, untranslated, all `--locked`.

## 8. Required documentation updates

None.

## 9. Acceptance criteria

- The test drives open → close → open through `App::update`, and sets no
  generation by hand.
- It was shown failing with the increment removed (scratch worktree), while the
  old test passed.
- No production change; all gates green.

## 10. Prohibited shortcuts

- Setting `similar_pairs_scan_generation` or `similar_pairs_scan` directly.
- Calling the handler function instead of `App::update`.
- The plant in the real working tree.
- Starting any arama binary except through `cargo test`.

## 11. Module boundaries

`app/src/core/tests.rs`.

## 12. Compatibility constraints

None.

## 13. Security constraints

None.

## 14. Known risks

- **The open event's route** (header message → app) may need the
  `Message::HeaderMessage(…)` wrapping. Use what the view would emit.
- **A real scan must not start.** If `App::update` turns out to execute anything
  synchronously, stop and report.

## 15. Required evidence

### 15.1 Evidence plan

| Question | Level | Evidence |
|---|---|---|
| Does the guard hold through the real open path? | logic and state | the new test |
| Can the test fail? | logic | its failure with `+= 1` removed, in scratch |

No GUI involved.

### 15.2 Deliverables

The rows above, observed, and the commit.

## 16. Review-request format

A package under `.git-exclude/review-request/` with a `## Entry point` section:
the commit, the `git show` command, and full repo-relative paths, one per line.
Report observed output; a check not run is recorded as not run. You cannot push:
report the commit, and the architect pushes.

## 17. Handoff checklist

| Required item | Covered in |
|---|---|
| Task title | Heading |
| Purpose | §1 |
| Background | §2 |
| Applicable RFC and requirements | §3 |
| Change scope | §4 |
| Explicit non-change scope | §5 |
| Required implementation | §6 |
| Required tests | §7 |
| Required documentation updates | §8 |
| Acceptance criteria | §9 |
| Prohibited shortcuts | §10 |
| Relevant code or module boundaries | §11 |
| Compatibility constraints | §12 |
| Security constraints | §13 |
| Known risks | §14 |
| Required evidence | §15 |
| Required review-request format | §16 |
