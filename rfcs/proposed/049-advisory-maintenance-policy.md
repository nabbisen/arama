# RFC 049: Advisory maintenance as a standing policy

**Status:** **Accepted by the project owner, 2026-10-24** (*"RFC 049 is
accepted."*). **No implementing task has been written** for §8's scheduled check
— acceptance of the policy is not authorization of the automation that enforces
it. The policy's §3 conditions and §6 filing rule apply from now regardless.
**Raised by:** T049's finding that a stated revisit condition sat satisfied for
66 days unnoticed (review 146), and the owner's question of 2026-10-24 — *"You
said T062 was non-RFC work. So what is it from and for?"*
**Relates to:** [RFC 027](../done/027-audit-warning-ledger-refresh.md)
(Implemented, 0.37.0), `rfcs/notes/audit-warning-burn-down.md`,
`.cargo/audit.toml`

## Summary

**The gate exists. The policy for keeping it green does not.**

`cargo audit --deny warnings` has been reconciled six times — Task 010, Task 044,
T049, T051, T058/T059, and now T062 — and each one arrived as a one-off task
with no governing document. [RFC 027](../done/027-audit-warning-ledger-refresh.md)
established the reconciliation and shipped in 0.37.0; it says nothing about
recurrence, and its status is Implemented, so nothing can be hung from it.

The owner's question exposed this. T062 is not work *without* provenance — it
executes a gate established by RFC 027 against a ledger that RFC 027 created.
What it lacks is a policy saying **when that execution is due, what makes it
correct, and who notices when it is overdue.** This RFC is that policy.

## 1. What exists today

| Piece | Where | Status |
|---|---|---|
| The gate | `.github/workflows/audit.yaml` — `cargo audit --deny warnings`, on push and PR to `main` | working |
| The ignore list | `.cargo/audit.toml` — 15 identities as of 2026-10-24 | working |
| The execution record | `rfcs/notes/audit-warning-burn-down.md` | working |
| The pairing rule | stated in `.cargo/audit.toml`'s own comment | working, by convention only |
| **A recurrence policy** | — | **missing, and this RFC** |

The ledger states the criterion that keeps this out of the RFC series:

> *"It is not an RFC because no product behavior, architecture policy, or public
> contract changed."*

That criterion is right and this RFC does not disturb it. §6 states what it means
for filing.

## 2. Three failure modes, all observed

### 2.1 A retired ignore goes unnoticed

Twice, and both were found by accident:

- **`rustybuzz`** — retired by snora 0.42.0's own dependency reduction. Noticed
  only while preparing Task 044.
- **`quick-xml`** — the ignore's comment named the exact condition that would
  retire it: *"when wayland-scanner publishes a version that accepts quick-xml
  0.41 or newer."* `wayland-scanner 0.31.11` shipped **2026-07-22**. Nothing
  acted until **2026-09-26** — 66 days, and only because T049 happened to
  re-read the comment.

**A revisit condition nobody re-reads is not a revisit condition.**

### 2.2 The gate is push-triggered, so silence is not green

`Audit` runs on push and PR. **It says nothing in between.**

Observed directly: green on **2026-10-06** (`4c722d8`), and red on the next push
**fourteen days later** (`b674239`, 2026-10-20). The advisories landed somewhere
in that gap and `main`'s true state was unknown for two weeks — not because the
gate failed, but because nothing asked it.

**A quiet fortnight is indistinguishable from a healthy one.** That is the
failure mode no amount of care in the existing gate can address.

### 2.3 Red arrives on commits that changed nothing

Four consecutive red runs on 2026-10-20/21 (`b674239`, `e1636c6`, `4fc444a`,
`b74b3fd`) — every one a documentation-only commit. Then again on 2026-10-23
(`c97e353`), also documentation-only.

The cost is not cosmetic. **A red gate blocks acceptance**: the architect cannot
distinguish a new break from the inherited one, so unrelated finished work waits.
T061 is waiting on T062 for exactly this reason, having nothing to do with
advisories.

## 3. What makes an ignore legitimate

Codified from what T059 did well, so it is a rule rather than a performance:

1. **The identity is listed in `.cargo/audit.toml` *and* has a matching entry in
   the ledger, in the same change.** An identity in one but not the other either
   leaves CI silently green on something untracked, or red on something the
   ledger does not explain.
2. **The dependency path is recorded**, not just the crate name.
3. **Reachability is determined, or explicitly recorded as undetermined.**
   T059's `zip` finding — that `candle-core` never calls `extract()` and is the
   only `zip` consumer in the lockfile — is the standard. A guess in either
   direction is worse than a stated unknown.
4. **The revisit condition is observable.** A version number, or a feature
   flag's presence in a manifest. **Never "when upstream fixes it"** — that is
   precisely the phrasing that let §2.1 happen twice.

## 4. Detecting a retired ignore

The mechanism, from review 146:

1. Copy `Cargo.lock`.
2. `cargo update` the copy, unconstrained.
3. `cargo audit` against the copy.
4. **Any currently-*ignored* identity that no longer appears has had its
   retiring condition met.**

This asks cargo and the advisory database the real question instead of inferring
it from prose. It needs no machine-readable schema and no third place to keep in
sync, and **it would have caught both of §2.1's misses** — each was exactly "a
dependency published a version that dropped the vulnerable transitive."

**Rejected alternative:** parsing each ignore comment's named blocking dependency
and re-resolving it. That is where the dev team's own sketch went, and it
requires the comments to become machine-readable, which adds a third artifact to
the pairing rule in §3.1.

## 5. Where it runs, and where it must not

**Not on push or PR.** A newly available upstream fix is not a defect in the
commit under test. Failing a contributor's PR because a crate published that
morning teaches people to re-ignore, which is the opposite of the intent.

**Scheduled, and at the release gate.**

- **Scheduled (weekly)** answers §2.2 as well as §2.1 — it asks the question when
  nobody is pushing, which is exactly when the answer is currently unknown.
- **At the release cut**, because that is already when the ledger is read.

Output is a **notice, not a failure**. The existing push gate keeps failing on
genuine new advisories; this one reports that an ignore may be retirable, which
is information, not a defect.

## 6. What this means for filing — the owner's question, answered by rule

Recurring execution under this policy is **task-shaped, not RFC-shaped**, by the
ledger's own criterion: it changes no product behaviour, no architecture policy
and no public contract. A dependency version bump to clear a gate is maintenance.

So:

- **A new advisory, or an ignore to add or retire → a task**, governed by this
  RFC, recorded in the ledger. No new RFC, and no handoff, because handoff status
  is inherited from an RFC and this policy is not per-instance.
- **A change to the gate, the pairing rule, or this policy → amends this RFC.**

That gives every future instance a governing document and makes the filing
question answerable by rule instead of by judgement each time — which is what the
owner asked for.

## 7. Non-goals

- **Automatically applying updates.** The check reports; a human decides. An
  auto-bump would land unreviewed lockfile changes on the one path where
  reachability analysis matters most.
- **Replacing `cargo audit` with `cargo-deny`.** The ledger already records why
  they differ — `cargo-deny`'s `unsound` scope excludes transitive dependencies
  by default and `cargo audit` has no such default, verified by un-ignoring
  `lru`. That is a separate decision, not this one.
- **Escaping unmaintained crates by vendoring or forking.** `zbus`/`zvariant`
  arrive through `iced`; that is upstream's to resolve.
- **Reducing the ignore list as a goal in itself.** T059 was explicitly told not
  to drop `pt2safetensors` to win an advisory count, and that judgement stands.

## 8. Acceptance

- A scheduled job runs the §4 detection weekly and reports retirable ignores as
  a notice, not a failure.
- The release checklist includes the same check.
- `.cargo/audit.toml` and the ledger are verifiably in sync — every identity in
  the array appears in the ledger **by identity**, checkable by script.
- §3's four conditions are stated where a future task will read them, not only
  here.

## 9. Open questions

- **Should the three older warning ignores be renamed by identity first?**
  `RUSTSEC-2025-0141`, `RUSTSEC-2024-0436` and `RUSTSEC-2026-0192` appear in the
  ledger only as crate names (`bincode` 2.0.1, `paste` 1.0.15, `ttf-parser`
  0.25.1), which defeats §8's scriptable check. Found in review 154; small, and a
  prerequisite rather than part of this.
- **Does the weekly notice go anywhere a person will see it?** A CI job that
  reports into its own log and nowhere else reproduces §2.1 in a new costume.
