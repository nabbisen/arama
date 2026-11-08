> **AUTHORIZED 2026-11-04 — owner:** *"Authorized."* Written 2026-11-04 by the
> architect.
>
> **Read RFC 049 §4.1, added 2026-11-08, before you start.** The mechanism you
> are building cannot see a retirement that needs a semver-major upgrade, and one
> such retirement exists *today* — `RUSTSEC-2025-0141`, retirable by moving
> `localcache` 0.21 → 0.26. **Your detector will, correctly, report nothing for
> it.** Do not widen this task to catch it; do say in the notice's own output
> that it covers semver-compatible updates only, so a quiet result is not read as
> "nothing retirable".
>
> **Do T065 first.** §5 requires `cargo audit --deny warnings` to exit 0 and
> `Audit` to be green with the notice reporting — neither is true until T065
> lands. Both tasks also edit `rfcs/notes/audit-warning-burn-down.md`, so they
> must not run concurrently.
>
> **Implements [RFC 049](../proposed/049-advisory-maintenance-policy.md)
> §4, §5 and §8.** Read §5 first — its trigger was **corrected on 2026-11-04**
> after the owner pointed out the project has no weekly routine, and the
> corrected version is what this task builds.
>
> **This adds CI automation, which is normally an owner exclusion.** The owner
> lifted it for this specifically, after `Audit` went red a third time on a
> commit that changed no code. **Do not extend the automation beyond §2.**
>
> **Moved 2026-11-08** from `.git-exclude/tasks/dev-team/` into `rfcs/handoffs/`, at the owner's direction, and completed against the 17-item Developer Handoff structure. The authorization above is unchanged.

# T066 Handoff — detect an ignore whose condition has already been met

Companion to [RFC 049](../proposed/049-advisory-maintenance-policy.md) §4, §5 and §8.

## 1. The one thing no existing gate can catch

`Audit` runs on push and PR and catches a **new** advisory. All three recent red
episodes surfaced that way, at the moment it had a cost.

**What it can never catch is a *retirable* ignore** — one whose revisit
condition has already been met. That never turns anything red; it just sits.
It has happened twice:

- **`rustybuzz`** — retired by snora 0.42.0, noticed by accident at Task 044.
- **`quick-xml`** — `wayland-scanner 0.31.11` shipped **2026-07-22**, acted on
  **2026-09-26**. **66 days**, with `docs/src/dev/release.md:36` *already*
  instructing someone to check exactly this by hand.

**That instruction is the proof the manual version does not work.** This task
replaces it with a command.

## 2. What to build — RFC 049 §4

```
1. Copy Cargo.lock.
2. cargo update on the copy, unconstrained.
3. cargo audit against the copy.
4. Any currently-IGNORED identity that no longer appears has had its
   retiring condition met. Report it.
```

It asks cargo and the advisory database the real question instead of parsing
prose, needs no schema, and **would have caught both misses above.**

**A script, in `.github/workflows/scripts/`** — `install-rust.sh` already
establishes that directory. It must be runnable by hand, because §3 puts it in
the release checklist where a person runs it.

**Never mutate the real `Cargo.lock`.** Work on a copy in a temp directory. A
detection check that leaves the lockfile updated would be a far worse defect
than the one it detects.

## 3. Where it runs — RFC 049 §5 as corrected

**On push, as a non-failing notice.** Add it to `audit.yaml` as a step *after*
the existing `cargo audit --deny warnings`, and **it must not be able to turn
the job red** — `continue-on-error`, or a guaranteed-zero exit. Write the
finding to `$GITHUB_STEP_SUMMARY` so it appears beside the CI result the pusher
is already reading.

**Why not a schedule:** no workflow here uses one, and the ROADMAP states
release prep is owner-driven with no set release point. A cron firing into an
Actions tab nobody opens is a signal with no reader — §2.1's failure mode again.

**At the release cut.** Replace `docs/src/dev/release.md:36`'s manual
*"confirm each `.cargo/audit.toml` ignore… still has a current… revisit
condition"* with an invocation of the script. Keep the sentence's intent; change
how it is satisfied.

## 4. The prerequisite — RFC 049 §9, and it comes first

RFC 049 §8 also wants `.cargo/audit.toml` and the ledger verifiably in sync,
**every identity in the array present in the ledger by identity**, checkable by
script.

**Three entries block that today.** `RUSTSEC-2025-0141`, `RUSTSEC-2024-0436` and
`RUSTSEC-2026-0192` appear in `rfcs/notes/audit-warning-burn-down.md` only as
crate names — the `### \`bincode\` 2.0.1`, `### \`paste\` 1.0.15` and
`### \`ttf-parser\` 0.25.1` headings at lines 116, 130 and 155. They are Task
044's, not a recent omission.

**If T067 has already landed, `bincode`'s entry will have moved to "Resolved"
and `RUSTSEC-2025-0141` left the ignore list** — name only the entries still
present.

**Add the identity to each heading or its first line.** Do not restructure the
entries or rewrite their reasoning — this is a naming fix so a script can match
them.

**Then add the sync check** to the same script: every identity in
`.cargo/audit.toml`'s `ignore` array appears in the ledger, and report any that
does not.

**Note for whoever writes the matcher:** the ledger groups identities in prose
— `RUSTSEC-2026-0338, 0339, 0340, 0341, 0342` and
`RUSTSEC-2026-0367 (…), 0368 (…), 0369 (…)`. A naive per-identity grep reports
false misses; the architect hit exactly that in review 154. **Expand grouped
suffixes, and verify your matcher against the current file, where the correct
answer is "every identity present" once §4's entries are named.** Do not
hardcode a number: T065 adds one identity and T067 retires one, so the count
depends on what has landed when you run it.

## 5. Verification

- **Run the script by hand** and show its output on the current tree. With
  nothing retirable it should say so plainly, not print nothing.
- **Prove it can detect.** Construct a case where an ignored identity is
  retirable — a scratch copy with an ignore for something a `cargo update`
  clears — and show the script reporting it. **A detector never shown detecting
  is not evidence.**
- **Prove it cannot fail the gate.** Show `Audit` green on a run where the
  notice has something to report.
- **Confirm the real `Cargo.lock` is unchanged** after a run — `git status`
  clean.
- The sync check reports **every identity in the array present** in the ledger —
  state the count you observed and which tasks had landed at the time.
- `cargo audit --deny warnings` still exits 0; the existing step is untouched.

## 6. Change scope

- `.github/workflows/scripts/` — the new script
- `.github/workflows/audit.yaml` — the non-failing step
- `rfcs/notes/audit-warning-burn-down.md` — §4's three headings
- `docs/src/dev/release.md` — line 36's manual instruction

## 7. Non-change scope

- the existing `cargo audit --deny warnings` step or `.cargo/audit.toml`
- any other workflow — §Banner
- any Rust source, `CHANGELOG.md`, or any `Cargo.toml` version field

## 8. Documentation updates

- **`docs/src/dev/release.md:36`** — replace the manual *"confirm each `.cargo/audit.toml` ignore…"* instruction with an invocation of the script, keeping its intent (§3).
- **`rfcs/notes/audit-warning-burn-down.md`** — name the older entries still present by identity, so the sync check can match them (§4).

## 9. Compatibility constraints

- The existing `cargo audit --deny warnings` step and `.cargo/audit.toml` stay untouched.
- **The new step must not be able to turn `Audit` red.**
- **The real `Cargo.lock` must never be modified** — the detector works on a copy in a temporary directory.
- Same runner and pinned `cargo-audit` version as `audit.yaml`.

## 10. Security constraints

- `audit.yaml` runs on `pull_request` as well as push, **so this step runs on contributors' pull requests**. It must use no secrets and write only inside a temporary directory.
- Treat `cargo update` and the advisory-database fetch as read-only network activity.
- Pin tool versions, as `audit.yaml` already does for `cargo-audit`.

## 11. Known risks

- **The semver limitation (RFC 049 §4.1).** A quiet result does not mean nothing is retirable; the notice must say it covers semver-compatible updates only.
- Grouped identities in the ledger cause false misses in a naive matcher (§4).
- **Do not hardcode an identity count** — T065 and T067 have already changed it.
- An unconstrained `cargo update` adds CI time to every push.

## 12. Acceptance criteria

RFC 049 §8, plus §5:

- The detection runs **on push as a non-failing notice** and cannot turn `Audit` red.
- The release checklist invokes it in place of `docs/src/dev/release.md:36`'s manual instruction.
- `.cargo/audit.toml` and the ledger are verifiably in sync **by identity**, checked by script.
- A detection is **demonstrated** on a constructed retirable case.
- The real `Cargo.lock` is unchanged after a run.

## 13. Required deliverables

Review request with `## Entry point`. Include the script's output on the current
tree, the constructed detection case from §5, proof the gate stays green, and
the sync check reporting every identity present, with the observed count.

You cannot push: report the commit and the architect pushes and watches every
workflow on that SHA.

Report observed output; a check not run is recorded as not run.

## 14. Handoff checklist

The 17 items the architect's operating instructions require of every Developer Handoff, and where each is covered here.

| Required item | Covered in |
|---|---|
| Task title | Heading |
| Purpose | §1 |
| Background | §1 |
| Applicable RFC and requirements | Authorization banner and companion line, §2, §3 |
| Change scope | §6 |
| Explicit non-change scope | §7 |
| Required implementation | §2, §3, §4 |
| Required tests | §5 |
| Required documentation updates | §8 |
| Acceptance criteria | §12 |
| Prohibited shortcuts | §2, §3, §7 |
| Relevant code or module boundaries | §6 |
| Compatibility constraints | §9 |
| Security constraints | §10 |
| Known risks | §11 |
| Required evidence | §5 |
| Required review-request format | §13 |
