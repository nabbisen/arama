# RFC 045: Automated Microsoft Store submission

**Status:** **Accepted by the project owner, 2026-08-25.** §12's gating
question remains open and is refined in §5.1.
**Depends on:** [RFC 042](./042-windows-store-distribution.md) (phase one — shipped
manually), [RFC 034](../done/034-release-workflow-reliability.md),
[RFC 037](../done/037-release-publication-atomicity.md),
[RFC 032](../done/032-cross-platform-external-ffmpeg.md)
**Requested by the project owner, 2026-08-25:** *"I would like to integrate the
publishment automation with GitHub Actions CI and Azure Entra ID."*

## Summary

arama is live on the Microsoft Store
([9nfcf9wn6z7s](https://apps.microsoft.com/detail/9nfcf9wn6z7s)), placed there by
hand. **Nothing in this repository builds an MSIX, and nothing submits one.**
The `packaging/windows/` directory holds an Appx manifest and four logos; the
release workflow builds five zips and a source archive and stops.

This RFC makes the Store package a **CI artifact built and verified by the same
workflow that builds the release assets**, and makes submission a **gated step
the owner approves**, using the Microsoft Store Developer CLI authenticated with
Microsoft Entra ID.

It answers RFC 042 §1 ("Does the Store package come from CI?") with *yes*, and
discharges RFC 042 §3b's binding requirement that whoever builds the submission
step must say how a failed or pending review surfaces.

## What is true today, stated as facts rather than recollection

| Fact | Established by |
|---|---|
| Store listing is live and free | `apps.microsoft.com/detail/9nfcf9wn6z7s` |
| No MSIX is built anywhere in CI | `.github/workflows/` — five workflows, none package |
| `packaging/windows/AppxManifest.xml` hardcodes `Version="0.41.1.0"` | the file, at `packaging/windows/AppxManifest.xml:9` |
| That version is two releases stale | current tag is `0.41.2` |
| **No GitHub Actions secrets exist on this repository** | `gh api repos/nabbisen/arama/actions/secrets` → `{"total_count":0,"secrets":[]}` |
| **No GitHub Environments exist** | `gh api repos/nabbisen/arama/environments` → empty |
| The CUDA variant cannot start without a CUDA install | import table of the shipped `0.41.2` binary, below |

The secrets line matters and is addressed in §7 — the credentials are not where
the workflow would need them to be.

## 1. The mechanism, verified rather than recalled

**`microsoft/store-submission@v1` is archived.** `gh api repos/microsoft/store-submission`
returns `archived=true`. Its README states existing workflows keep running but
no fixes, features or security updates will be published, and directs users to
the MSStore CLI. **Do not use it.** It is the first result for most searches on
this subject and is the likely wrong turn.

**The current path is the MSStore CLI**, placed on `PATH` by
`microsoft/microsoft-store-apppublisher`. That repository is active
(`archived=false`, pushed 2026-08-24); its latest release is **`v1.4`, published
2026-07-21**. Microsoft's own documentation still pins `@v1.1` — **pin `v1.4`**
and record that the doc lags.

Two prerequisites are already satisfied and one is not:

- ✅ *"The app you want to update must already be published and live"* — it is.
- ✅ *"Supported for free products only. Paid products will be supported in a
  future release."* — arama is free. **This is a live constraint, not a
  footnote:** the day arama is ever priced, this automation stops working.
- ❓ The Entra ID application must be added in **Partner Center → Account
  settings → User management → Microsoft Entra applications** and assigned the
  **Manager** role. Registering the app in Entra alone is not sufficient; the
  Partner Center association is the half that grants it submission rights.

The four credentials, with the secret names Microsoft's documentation uses:

```
AZURE_AD_TENANT_ID              Entra → Identity → Overview → Tenant ID
AZURE_AD_APPLICATION_CLIENT_ID  Entra → App registrations → Application (client) ID
AZURE_AD_APPLICATION_SECRET     Entra → App registrations → Certificates & secrets
SELLER_ID                       Partner Center → Account settings → Identifiers
```

## 2. Which binary goes in the package — settled by measurement

**The CPU variant. Not the CUDA variant.** This was raised by the owner on
2026-08-24 and answered then; it is re-confirmed here against the *shipped*
`0.41.2` artifacts because it is now a constraint the automation must encode
rather than a judgement someone re-makes.

`arama@Windows-x64-gpu-cuda-0.41.2.zip` contains one file, `arama.exe`, whose
import table carries:

```
cublas64_13.dll
curand64_10.dll
```

They are in **the normal import table**, not delay-loaded — the Delay Import
Directory is all zeros and the binary has no `.didat` section. Windows resolves
them in the loader **before `main` runs**, so on a machine without them the
process fails to start with a loader error and arama's own fallback never
executes.

**An NVIDIA driver alone does not supply them.** cuBLAS and cuRAND ship with the
CUDA Toolkit redistributables, so the package would fail to launch for most
NVIDIA owners as well as for everyone else. `arama@Windows-x64-cpu-0.41.2.zip`
imports neither.

This is RFC 042's whole CPU/GPU problem arriving in the Store channel, and until
Phase 0's two `[patch.crates-io]` entries or an upstream candle change land, the
Store gets the CPU build. **The workflow must name the CPU artifact explicitly
and fail if it is absent** — not glob for "the Windows zip".

## 3. Two halves, separable on purpose

**Half A — build and verify the MSIX. Fully automatic, no credentials, no
outward effect.** Runs on every tag, in `release-executable.yaml` alongside the
existing jobs. Produces `arama-<tag>.msix` and attaches it to the GitHub
release as a seventh asset.

**Half B — submit it. Gated on the owner's approval, holds the credentials.**
A separate job that cannot start until a human approves it.

The split is the point. Half A is deterministic, reviewable, and safe to run
unconditionally; it also means that even if submission never runs, the Store
package is always built and always proven, so a manual submission is a file
download rather than a packaging session. Half B is the only part that touches
the outside world.

RFC 034's lesson applies to Half A: an operator-driven channel silently produced
nothing, so the packaging must be CI's job. RFC 037's lesson applies to Half B:
nothing is published until every prerequisite has actually succeeded.

## 4. Half A — packaging, and the version that cannot go stale

`makeappx` is present on `windows-latest`. RFC 042 §3 already proved the
mechanics in 39 seconds on the first attempt, and its notes stand:

- **Glob the SDK `bin` directory** for `makeappx`/`signtool`; do not hardcode a
  build number.
- **`AllowAllTrustedApps` is unset on the runner** and must be enabled for the
  install step.
- **Launch via `shell:AppsFolder\<PackageFamilyName>!App`**, never the raw
  executable — a direct invocation runs without package identity and tests
  nothing while appearing to pass.

### 4.1 The version becomes structurally underivable-from-stale

RFC 042 §4 settled `X.Y.Z` → `X.Y.Z.0`, derived at build time, storing nothing
new. The committed manifest violates that today by hardcoding a value, and the
value is wrong — which is exactly the failure mode the rule existed to prevent.

**Set the committed value to `0.0.0.0` and have CI overwrite it.** A placeholder
that is obviously not a release cannot be mistaken for one, and CI substituting
it means a stale number can never reach the Store:

```powershell
[xml]$xml = Get-Content packaging/windows/AppxManifest.xml
if ($xml.Package.Identity.Version -ne '0.0.0.0') {
  throw "AppxManifest Version must stay 0.0.0.0 in the repository; CI derives it."
}
$xml.Package.Identity.Version = "$env:TAG.0"
$xml.Save("$staging/AppxManifest.xml")
```

The guard is the load-bearing half. Without it the placeholder is a convention;
with it, it is enforced.

**RFC 034 Part F** compares the Cargo manifest version to the tag. Its
comparison must now append or strip `.0` symmetrically when checking the Appx
manifest — a comparison change, not a second recorded version.

### 4.2 The package is smoke-tested before anyone is asked to approve it

Sign with a throwaway self-signed certificate, install, launch through
`AppsFolder`, confirm it runs, then **submit the unsigned package** — the Store
re-signs, so the test certificate never leaves CI.

Three checks the packaged build must pass, each of which already has a contract
behind it:

- **RFC 032** — no `ffmpeg`/`ffprobe` inside the `.msix`. The absence contract
  does not stop at the GitHub channel.
- **RFC 041** — data locations resolve to the packaged app's own directories.
  This is the `NATIVE_SMOKE_DATA_LOCATIONS_RESOLVED` check, and packaged
  identity redirects `AppData`, so this is the environment where RFC 041 is most
  likely to be wrong.
- **Task 037** — `Subsystem 00000002 (Windows GUI)` on the binary inside the
  package, so a console window cannot reappear through the Store channel
  specifically.

## 5. Half B — the approval gate, which resolves a standing conflict

**There is a conflict here and I am not resolving it alone.** The owner's
standing rule is that tags, releases, outbound correspondence and *anything
published under the owner's name* require authorization every time. A workflow
that submits to the Store automatically on every tag would violate that rule by
construction — the Store listing is published under the owner's name, and Store
review is the one gate this project does not control.

RFC 042 §3b, meanwhile, settled that **every GitHub release also goes to the
Store, "for now"**, with the qualifier recorded as load-bearing.

**These are compatible if "automatic" means everything up to the submission, and
the submission itself waits for one click.** The mechanism is a GitHub
Environment with a required reviewer:

```yaml
store-submit:
  needs: [release, package-msix]
  runs-on: windows-latest
  environment: microsoft-store      # protection rule: required reviewer = nabbisen
  steps:
    - uses: actions/checkout@v4
    - uses: microsoft/microsoft-store-apppublisher@v1.4
    - name: Configure store credentials
      run: msstore reconfigure `
             --tenantId ${{ secrets.AZURE_AD_TENANT_ID }} `
             --sellerId ${{ secrets.SELLER_ID }} `
             --clientId ${{ secrets.AZURE_AD_APPLICATION_CLIENT_ID }} `
             --clientSecret ${{ secrets.AZURE_AD_APPLICATION_SECRET }}
    - name: Publish package
      run: msstore publish "${{ github.workspace }}/dist/arama-${{ github.ref_name }}.msix" -id 9NFCF9WN6Z7S
```

The job appears in the run, pauses, and emails the owner; it starts only when
approved and expires if it is not. **Authorization is asked for every time, it
is asked for in one place, and it is recorded** — which is stronger than the
current arrangement, where the same authorization is given in conversation and
recorded nowhere.

**This also scopes the credential.** Environment secrets are readable only by
jobs that declare that environment, so no other workflow — and no future
workflow added carelessly — can read the Entra client secret. A repository-level
secret would be readable by all five existing workflows. **Use environment
secrets, not repository secrets**, and this is a security decision rather than a
stylistic one.

### 5.1 Gating is a repository setting, not a code path

The owner asked whether gating is "a bit far from automation". It is worth
being exact about what it costs, because the answer is smaller than it looks
and the choice is cheaper to change than a design decision usually is.

**Every mechanical step is automated either way** — selecting the CPU artifact,
deriving the version, packaging, signing, installing, launching, the three
contract checks, attaching the asset, authenticating to Entra, uploading,
submitting. What a gate adds is one approval click on a run the owner is already
watching, because tagging is already an attended operation in this project.

**And the two forms are the same workflow.** `environment: microsoft-store` must
stay regardless — it is what scopes the Entra secret to the one job that may
read it. The gate is the environment's *required-reviewer protection rule*, set
in repository settings. Turning it off is a checkbox; nothing in
`.github/workflows/` changes, and no release is re-cut to switch.

**So this need not be settled once and for all.** The argument for gating the
first submissions is not caution in principle — it is that the credentials have
never authenticated, the failure shapes are unknown, and §6's certification
blind spot is real. Those are all conditions that expire. **Recommended: gate
the first two or three submissions, then drop the protection rule** once the
channel has demonstrably worked, leaving the scoped secret in place.

There is also a genuine argument for automatic from the start, and it deserves
recording rather than dismissing: the tag push is already the authorization for
the GitHub release, which goes fully public with no second click. By that
precedent the Store submission is downstream of an authorization that has
already been given. The distinction is that a Store submission consumes an
external review cycle and is harder to retract — which is a reason to watch the
first few, not a reason to click forever.

**Recommendation: the gated form.** If the owner prefers fully automatic
submission with no click, that is theirs to decide and it is a deliberate
narrowing of the standing rule — but it should be said explicitly and recorded
here, not arrived at by my writing a workflow that assumes it.

## 6. How a failed or pending review surfaces — RFC 042 §3b's requirement

**What "certification" means here, stated plainly.** Sending a package to the
Store is two events, not one, and only the first is ours:

1. **Submission.** `msstore publish` uploads the package and Microsoft accepts
   it into the queue. This is what returns success to the workflow, and it means
   *received*, not *approved*.
2. **Certification.** Microsoft then scans and reviews the package — malware
   checks, manifest validation, policy compliance, sometimes a human. It takes
   hours to days, on Microsoft's schedule. It ends in the update going live, or
   in a **rejection with reasons**.

**Nothing in this repository observes the second event.** So a rejected
certification looks exactly like a successful release from inside arama: the
workflow is green, the tag exists, the GitHub release is published, the
CHANGELOG says shipped — and the Store quietly keeps serving the previous
version. The only way anyone finds out is by opening Partner Center and looking.

That is the same shape as the failure RFC 034 was written for: a channel that
silently produces nothing while presenting as success. It is why this section
exists, and why it is the part of the design I am least satisfied with.

RFC 042 §3b named this and refused "someone will notice" as an answer. It is
still the weakest part of the design and the honest position is that **this RFC
improves it without closing it.**

What the gate buys: submission failures — bad credentials, a rejected package, a
malformed manifest — become a **red job on a run the owner already approved**,
rather than silence. That covers the submission call.

What it does not buy: **certification happens after `msstore publish` returns
successfully**, on Microsoft's schedule, and nothing here polls it. A package
that submits cleanly and fails certification two days later still surfaces only
in Partner Center.

**Proposed floor, not a solution:** the submit job writes the submission
response — including the submission id — into the run log and the job summary,
so there is a durable, timestamped record of what was submitted and when. Then a
follow-up decision, deliberately deferred: `msstore submission status` on a
schedule, opening an issue on a non-terminal state. That is worth doing only
once the automation has run a few times and the failure shapes are known, and I
would rather propose it from evidence than guess at it now.

## 7. The blocker: the credentials are not in GitHub

The owner reported the Entra ID secrets are published. **They are not on this
repository** — `gh api repos/nabbisen/arama/actions/secrets` returns
`{"total_count":0,"secrets":[]}`, and there are no environments, checked with a
token carrying `repo` scope. The most likely reading is that the Entra app
registration exists and the values are in hand, but the GitHub half has not been
done; the second most likely is that they were added somewhere this repository
cannot see.

**Nothing about this is a problem** — it is the ordinary next step, and it is
the owner's to do because it requires Partner Center and Entra portal access. It
is recorded because building a workflow against secrets that do not exist would
produce a green-looking design and a red first run.

What is needed, once §5's shape is chosen:

1. Create the environment `microsoft-store` with **required reviewer: nabbisen**.
2. Add these four secrets **to that environment**:
   - `AZURE_AD_TENANT_ID`
   - `AZURE_AD_APPLICATION_CLIENT_ID`
   - `AZURE_AD_APPLICATION_SECRET`
   - `SELLER_ID`
3. Confirm the Entra application is listed in **Partner Center → Account
   settings → User management → Microsoft Entra applications** with the
   **Manager** role.

## 8. The Appx manifest divergences — verify, do not assume

`packaging/windows/AppxManifest.xml` differs from the manifest RFC 042 §3 proved
on a runner. **A package built from the committed manifest is live on the Store,
so it evidently certified** — these are therefore flagged for verification, not
asserted as defects:

- **`MinVersion="10.0.17763.0"`.** RFC 042 §3 records that
  `uap10:TrustLevel="mediumIL"` with `rescap:runFullTrust` requires
  `10.0.19041.0`. If that is right, the package declares support for builds it
  may not run on — a runtime failure on 17763–18362, not a certification one.
- **`MaxVersionTested="10.0.19041.0"`** where the proven manifest used
  `10.0.22621.0`.
- **`EntryPoint="Windows.FullTrustApplication"`** where the proven manifest used
  `uap10:RuntimeBehavior="packagedClassicApp"`. The committed file carries
  `uap10:TrustLevel` without the `uap10:RuntimeBehavior` it normally pairs with.
- **`ProcessorArchitecture` is absent from `<Identity>`**, so `makeappx` will
  produce a `neutral` package for a payload that is x64-only. Set `x64`.

§4.2's install-and-launch step is what turns each of these from an opinion into
a result, which is why it belongs in Half A rather than in a review comment.

## 9. Non-goals

- **Metadata automation.** Listing text, screenshots and descriptions stay
  manual. The `msstore submission updateMetadata` path exists and is a separate
  decision; nothing currently argues for taking the store listing's prose out of
  the owner's hands.
- **Publishing the CUDA variant to the Store.** §2.
- **Store review polling.** §6, deliberately deferred.
- **Changing what the GitHub release contains.** The `.msix` is added; the six
  existing assets and RFC 030's naming are untouched.
- **Signing for distribution.** The Store signs. The self-signed certificate in
  §4.2 exists only to make the package installable for its own smoke test.

## 10. Testing and verification

- The `.msix` installs and launches through `AppsFolder` on a clean runner.
- The three packaged-build checks in §4.2, each reported as observed output.
- The version guard rejects a manifest whose committed version is not `0.0.0.0`
  — proven by a deliberate failing case, not by the happy path alone.
- A submission dry run against the real credentials **before** the first gated
  release, so the first live use is not also the first authentication attempt.

## 11. Risks

- **The credential is the highest-value secret this project has held.** It can
  publish to the Store under the owner's identity. Environment scoping (§5) is
  the mitigation; the residual risk is that anyone with write access can approve
  their own submission.
- **Store review remains an uncontrolled external gate.** §6 narrows the blind
  spot without removing it.
- **`microsoft-store-apppublisher` is a third-party dependency in the release
  path**, pinned at `v1.4`. It is Microsoft's own, but the previous official
  action was archived — pin it, and expect to move again.
- **Free-products-only.** Pricing arama breaks this automation. Recorded in §1.

## 12. Open questions — owner-reserved

- **§5: gated submission, or fully automatic?** Recommended: **gated for the
  first two or three submissions, then flip.** Per §5.1 this is a repository
  setting rather than a code path, so it is reversible at any time without
  touching the workflow or re-cutting a release. Still the owner's to settle,
  and still the standing-directive conflict — but not a decision that has to
  hold forever.
- **Does the Store listing lag matter enough to submit on every tag?** RFC 042
  §3b said yes "for now"; the gate makes reversing it free, since not approving
  is the same as not submitting.
