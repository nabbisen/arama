# The RFC portfolio, reviewed against the owner's design philosophy

**Date:** 2026-10-21
**Reviewer:** architect
**Scope:** the five proposed RFCs — 042, 045, 046, 047, 048

The owner stated two principles as fundamental and asked that the portfolio be
judged against them:

1. **"Finally clean, safe and secure, and robust and sophisticated design."**
2. **"APIs and UI/UX for users not to be confused or misunderstand are also very
   important."**

The word doing the work in (1) is **"finally"** — the end-state design, not the
expedient patch. (2) makes clarity a requirement rather than polish: a name that
does not mean what it says, a control that is available before it works, a
silent truncation, or a document describing behaviour the code does not have are
**defects**, not cosmetics.

This note records what changes when the portfolio is read that way. It does not
re-litigate any RFC's technical analysis.

---

## The headline: the philosophy re-ranks the portfolio

The RFCs are currently prioritised by **audit severity**. Under principle 2 a
different grouping appears, and it cuts across three RFCs:

| Where | The user is told… | The truth |
|---|---|---|
| RFC 047 | Stop interrupts indexing — in four documents | The thumbnail phase cannot be aborted at all |
| RFC 047 | The Similar Pairs button greys out until indexing finishes | It ungreys ~35 lines before embedding is even spawned |
| RFC 048 | "These are the similar pairs" | These are the first 50, if images; all of them, if videos |
| RFC 048 | (file order is presented without comment) | Hash order — stable within a run, arbitrary between runs |
| RFC 046 | Video similarity uses wav2vec2 embeddings | Early convolutional features, mean-pooled twice; 95.1 % of the shipped model is never read |

**Every row is the application misdescribing itself to its user.** That is a
single coherent theme, it is exactly principle 2, and it is currently spread
across three separately-prioritised documents. It is the most
philosophy-aligned batch of work available.

---

## RFC 046 — audio model fidelity

**Principle 1 narrows the route choice, and the RFC should say so.**

§3 presents three routes as a technical decision: implement the transformer,
swap the model, or restate the documentation to match what runs. Under
"**finally** clean", the third is the expedient patch by definition — it changes
no behaviour and leaves 605 MB shipped of which **95.1 % is never read**. A
design that carries twenty times the weights it uses is not clean under any
reading of the word.

The owner has already said as much in substance — *"We can revise specs and
switch models if it becomes better. I want finally clean, safe and secure, and
robust and sophisticated design"* — but RFC 046 still presents restate as an
equal third option. **Recommendation: record in §3 that restate-only is excluded
by the owner's standing principle**, leaving a genuine two-way choice between
implementing what we ship and shipping something we fully use.

**Principle 2 independently supports acting.** The documentation says wav2vec2
embeddings; the code computes something materially different. A user comparing
videos cannot know what "similar" means here. That is a misdescription defect in
its own right, and it holds whichever of the two remaining routes is taken.

---

## RFC 047 — indexing cancellation

**This RFC is principle 2 in its purest form.** Its own opening sentence —
*"Four documents describe an application that can be interrupted. It cannot."* —
is the principle restated as a finding.

**Principle 1 is well served too.** `spawn_blocking` is characterised as *"not a
technique to introduce, it is one to apply consistently"*, with two existing
call sites named. That is end-state reasoning rather than a local patch.

**Two gaps worth closing:**

**§7's open question should be decided, not inherited.** Whether cancellation is
cooperative-and-partial or transactional has *"never been stated as a
decision"*, and RFC 015's cache lifecycle does not cover it. Under "finally
clean", cache state after a Stop should be a decision with a reason, not the
behaviour that happens to fall out of the implementation. The likely answer —
partial, because a partial index is useful and resuming is cheap — is probably
right; it needs writing down, not discovering.

**The pair scan is split across two RFCs in a way that can ship half a fix.**
RFC 047 §3.3 takes cancellation and progress and defers the *bound* and the
retained-versus-total counts to RFC 048 §3. Those are two halves of one
user-visible behaviour. Shipping 047 alone gives a pair scan that can be
stopped and still lies about how many results it found. **Recommendation: if
they ship separately, 048 §3 must not slip past 047** — or move §3 into 047,
since it is a clarity fix and 047 is the clarity RFC.

---

## RFC 048 — library scale

**The strongest principle-2 content in the portfolio**, and §3 is the sharpest
item in it. A silent cap is *"indistinguishable from completeness"*, and the RFC
correctly connects that to RFC 036, which built absence states precisely so
"nothing found" could never be read as "something broke". A silent truncation
reintroduces the same ambiguity one level up.

I re-verified the asymmetry: `MAX_IMAGE_SIMILAR_PAIRS = 50` exists at
`similar_pairs_dialog.rs:33` and **no video equivalent exists anywhere in the
tree**. Images are capped, videos are not, and the user is told neither.

**Principle 1 is served by §1's framing**, which is the most quietly damning
sentence in the portfolio: the gallery's per-frame widget construction imposes a
library-size ceiling that has been *"turned into a setting and then into
advice"* — the depth-2 cap, then the FAQ telling users to reduce scope. **A
design limit has been patched around three times without being fixed once.**
That is precisely what "finally" exists to prevent.

**One re-ranking note.** 048 is marked High alongside 047, but §2 and §3 — the
meaningless order and the silent truncation — are small, self-contained, and are
the two items a user is most likely to be actively misled by. They do not need
to wait for §1's virtualisation work. **Recommendation: consider splitting 048
so §2 and §3 can land early**, with §1 and §4 (the scaling work) following.

---

## RFC 042 — Windows Store distribution

**Principle 1 already decides more of this than the RFC acknowledges.** Option A
(bundle both binaries behind a selector) is described, by the RFC's own
analysis, as *"introduc[ing] a failure mode that does not exist today"* and
*"creat[ing] a fourth distribution shape"* whose assembly nothing in CI
verifies. Both are direct arguments against it under "safe and robust". The RFC
reaches that conclusion in substance; the principle makes it explicit.

**A present-tense principle-2 defect is buried here as a design question.**
§Design question 2 asks whether GPU use should be a user setting. Verified
today:

```rust
// crates/ai/src/model/model_manager.rs:13
Device::new_cuda(0).unwrap_or(Device::new_metal(0).unwrap_or(Device::Cpu))
```

**Nothing in `crates/ui/` or `app/src/` surfaces which device was chosen, and
no i18n key mentions it.** So:

- A user cannot find out whether their GPU is being used.
- A CUDA initialisation *failure* is indistinguishable from *no GPU present* —
  both are swallowed by `unwrap_or` and silently become CPU.
- The only symptom is that indexing is slower than expected, with nothing to
  attribute it to.

That is a silent degradation path and an unanswerable user question, and it
exists **now**, independently of whether arama is ever bundled for the Store.
**Recommendation: lift it out of RFC 042 as a defect in its own right.** It is
small, it is pure principle 2, and it is currently owned by an RFC that is
blocked on an unanswered upstream candle PR — so as things stand it ships
nothing.

---

## RFC 045 — automated Store submission

Least principle-2 surface, because its user is the owner rather than an end
user — but it treats that user correctly. §6 discharges a binding requirement to
say **how a failed or pending review surfaces**, which is the same clarity
principle applied to an operator.

**Principle 1 is served by §5.1's observation** that gated-versus-automatic is a
repository setting rather than a code path, so the decision is reversible
without touching the workflow or re-cutting a release. A design whose governance
choice can be changed without a code change is the robust shape.

No change recommended. It is blocked on credentials, which is an owner action,
not a design question.

---

## Summary of recommendations

1. **RFC 046 §3** — record that restate-only is excluded by the owner's standing
   principle; the choice is implement-or-swap.
2. **RFC 047 §7** — decide the partial-versus-transactional question rather than
   inheriting it.
3. **RFC 047 §3.3 / RFC 048 §3** — do not let the pair-scan bound ship later
   than its cancellation; consider moving §3 into 047.
4. **RFC 048** — consider splitting, so §2 (deterministic order) and §3 (honest
   counts) land ahead of §1's virtualisation.
5. **RFC 042** — lift the silent CPU/GPU selection out as a defect of its own,
   so it is not blocked behind an upstream PR nobody has answered.

Items 1–4 are amendments to accepted analysis, not new proposals. **Item 5 is
the only one that would create new work**, and it is the smallest item in this
note.
