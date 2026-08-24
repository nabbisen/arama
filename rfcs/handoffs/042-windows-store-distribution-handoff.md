# RFC 042 Handoff — Windows Store distribution, phase one

Companion to [RFC 042](../proposed/042-windows-store-distribution.md), accepted
by the project owner 2026-08-16. It stays in `rfcs/proposed/` until the work
ships, per [RFC 000](../done/000-rfc-lifecycle-policy.md).

**This handoff covers only what does not depend on the route.** The choice
between one binary and Option A's bundle is waiting on
[candle#3900](https://github.com/huggingface/candle/issues/3900) by the owner's
decision, and **nothing here forces it.**

**Read the RFC first**, and read Task 026's result — the capability question was
settled by execution, not argument, and re-litigating it would waste a cycle.

## 1. Design authority

1. [RFC 042](../proposed/042-windows-store-distribution.md);
2. `.git-exclude/reviewed/110-task026-msix-capability-probe-result-review.md`
   and RFC 042 §3's own boxed answer — **a full-trust MSIX reads arbitrary
   paths with no capability declared, proven on `windows-latest`.** Do not
   declare `broadFileSystemAccess`;
3. [RFC 034](../done/034-release-workflow-reliability.md) and
   [RFC 037](../done/037-release-publication-atomicity.md) — why the release
   channel looks the way it does, and what a silent no-op costs;
4. [RFC 030](../done/030-distribution-and-version-contracts.md) — the asset
   naming and layout contracts a Store package must not quietly break.

## 2. What is settled and must not be reopened

- **No `broadFileSystemAccess`.** Task 026 ran it: `MSIX_PROBE_READ_DIR=Ok`,
  `MSIX_PROBE_FILE_CONTENT=msix-capability-probe-marker-value`, with no
  capability declared. Read-directory and read-file are separable permissions
  and both succeeded.
- **`uap10:TrustLevel="mediumIL"` + `rescap:runFullTrust`**, requiring
  `MinVersion="10.0.19041.0"`.
- **Version: `X.Y.Z` → `X.Y.Z.0`**, derived at package-build time. Store
  packages take four parts and **the fourth is reserved for Store use and must
  be zero at build time.** Store nothing new — RFC 034 Part F's
  manifest-equals-tag check gains a symmetric append-or-strip of `.0`, which is
  a comparison change, not a second recorded version.
- **The mechanics Task 026 paid for**, all in RFC 042 §3: `AllowAllTrustedApps`
  is unset on `windows-latest` and must be enabled; glob the SDK `bin` directory
  rather than hardcoding a build number; **launch via
  `shell:AppsFolder\<PackageFamilyName>!App`**, never the raw executable, or the
  process runs without package identity and proves nothing while appearing to
  pass.

## 3. The credentials — read this before writing any workflow

The owner has obtained **Entra ID credentials for Microsoft Store submission.**

**This is the most dangerous thing this repository has ever been near.** It is
not a build token or a read key: it publishes software to end users under the
owner's name and Microsoft's imprimatur. A leaked release token lets someone
ship arama to arama's users.

**Requirements, none of them negotiable:**

- **GitHub Actions encrypted secrets only.** Never in the repository, never in
  a workflow file, never in a committed `.env`, never in `.git-exclude/` —
  gitignored is not the same as safe, and a file on disk outlives the intent
  that created it.
- **Never echoed.** No `set -x` around them, no `echo` for debugging, no passing
  them as command-line arguments where they land in a process list. Actions
  masks known secret values in logs; it cannot mask a value you derived from one.
- **Scope the workflow's own permissions.** The job that submits needs nothing
  else — no `contents: write`, no package registry.
- **The submitting job runs only from a tag**, never from a pull request.
  A PR from a fork must not be able to reach a Store credential, and `on:
  pull_request` plus a secret is the standard way that goes wrong.
- **Say what happens when the credential expires**, because Entra ID secrets do.
  A submission that silently stops working is exactly the 0.37.0 failure shape
  in a new place — record the expiry and how a failure surfaces.

**If any of this cannot be satisfied, stop and report it.** A working submission
pipeline built on a credential handled loosely is worse than no pipeline.

## 4. The question the owner must answer before submission is wired

**Does every GitHub release also go to the Store, or only some?**

Automated submission makes "every tag submits" the default, and **Store review
is an external gate on a schedule this project does not control.** RFC 042 §Risks
already names it: *"Nothing here should be sequenced as though approval is
certain."*

Consequences worth putting in front of the owner rather than choosing for them:

- **Every release:** the Store listing tracks GitHub closely; a rejected or slow
  review sits behind every subsequent release and someone has to notice.
- **Selected releases:** the Store lags deliberately, the listing shows a
  version older than GitHub, and something must decide which — a manual
  dispatch, a tag convention, a label.

**Do not pick one.** Build the pipeline so either is a small change, and put the
question to the owner with what you learned building it.

## 5. Phase one — what to build

**5.1 Package identity, and it needs the owner.** Reserving a name in Partner
Center is an account action, not a code action. **Find out what identity values
the manifest needs** — publisher, package family name, identity name — and
report them; do not reserve anything.

**5.2 A packaging job in CI.** RFC 042 design question 1's recommendation
stands: *whatever is submitted is built and verified by the same workflow that
builds the assets.* RFC 034 exists because an operator-driven channel silently
produced nothing.

**Do not extend `release-executable.yaml` on your first attempt.** That workflow
publishes five assets and took four tag pushes and three defects to stabilise.
Build the packaging separately, prove it, and propose the integration as its own
step with its own review.

**5.3 The package contents are the route's business, not yours.** Build the
pipeline around **one** Windows executable for now — the CPU variant is the
obvious placeholder — and structure it so that swapping in a second binary plus
a selector is a contents change rather than a rewrite. **Say clearly in the
package what would have to change under Option A.**

**5.4 Verify the packaged application actually runs**, the way Task 026 did:
sideload, launch via shell activation, confirm it starts. A package that builds
is not a package that runs, and this project has shipped a release with five
assets and zero of them attached.

## 6. Non-scope

- **The route.** §Options and Phase 0 in the RFC. Waiting on candle by the
  owner's decision.
- **Actually submitting anything to the Store.** Phase one builds and proves a
  package; submission is a separate authorization and probably a separate task.
- **Reserving the listing name**, or any Partner Center account action. §5.1.
- **GitHub release distribution.** Unchanged, and RFC 042 says so.
- **macOS or Linux packaging.**
- **A GPU setting** (RFC 042 design question 2) — only Option A forces it, and
  Option A is not chosen.
- **Write and delete under MSIX.** Untested, named in RFC 042 §3, and arama
  ships `file-handle` with `trash`. **Not in scope, and not to be assumed
  working** — if phase one happens to establish it, report it.

## 7. Acceptance criteria

- An MSIX package builds in CI from the same source the release assets are
  built from.
- **No capability declared**, verified by grep of the generated manifest, not by
  intent.
- The four-part version is derived at build time from the manifest version, and
  RFC 034 Part F's check still passes with the `.0` handling symmetric.
- The packaged application **installs and launches** via shell activation, with
  the observed output shown.
- Credential handling per §3, or a clear report of what could not be met.
- §4's question put to the owner with what the work taught you about it.
- Under Option A, what would change — stated, not implied.
- Gates clean, exit codes captured.
- **No `CHANGELOG.md` entry** unless something user-visible ships, which in
  phase one it should not. Say which.

## 8. Required deliverables

A review request under `.git-exclude/review-request/NNN-<slug>/` opening with an
`## Entry point` section giving the pinned commit, the exact `git show` command,
and plain paths to every file. Include the manifest, the workflow, the install
and launch evidence, and §5.1's identity findings.

Report observed output; a check not run is recorded as not run. **A credential
you could not verify is recorded as unverified, never as working.**
