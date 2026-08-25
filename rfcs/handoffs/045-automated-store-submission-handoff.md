# RFC 045 Handoff — Half A: build and verify the Store package in CI

Companion to
[RFC 045](../proposed/045-automated-store-submission.md), accepted by the
project owner 2026-08-25.

**Read the RFC first.** This handoff does not restate it; it settles what the
RFC left open for the implementer and names the traps.

## 1. Scope — Half A only. Do not build the submission job.

RFC 045 splits the work in two. **This handoff is Half A: producing a verified
`.msix` as a release asset.** It requires no credentials, contacts nothing
outside the runner, and cannot publish anything.

**Half B — `msstore`, Entra ID, the approval gate — is explicitly out of scope
and must not be started.** Two things it depends on do not exist yet: the four
secrets are not on the repository (`total_count: 0`, verified), and the owner
has not settled RFC 045 §12's gating question. Writing it now would mean writing
against credentials nobody can test with.

This split is not bureaucratic. RFC 042 §1's recommendation was that whatever is
submitted is *built and verified by the same workflow that builds the assets,
even if submission itself stays manual* — Half A alone satisfies that, and turns
the owner's manual submission from a packaging session into a file download.

## 2. Design authority

1. [RFC 045](../proposed/045-automated-store-submission.md) §§2, 4, 8;
2. [RFC 042](../proposed/042-windows-store-distribution.md) §3 — the packaging
   mechanics, **already proven on a runner**. Read the "Mechanics worth not
   rediscovering" block before writing a line of YAML; it will save you the four
   failures it lists;
3. [RFC 034](../done/034-release-workflow-reliability.md) Part F — the
   manifest-equals-tag gate this work extends;
4. [RFC 032](../done/032-cross-platform-external-ffmpeg.md) and
   [RFC 041](../done/041-application-data-locations.md) — two contracts the
   packaged build must be shown to keep.

## 3. The CPU variant. Name it; do not glob for it.

RFC 045 §2 settles which binary is packaged and why. The constraint for you is
narrow and absolute:

**The `.msix` payload is the `x86_64-pc-windows-msvc` CPU build.** Not the CUDA
build, not "the Windows artifact", not the first zip matching a pattern.

`arama@Windows-x64-gpu-cuda-*.zip` hard-imports `cublas64_13.dll` and
`curand64_10.dll` — normal import table, empty delay-import directory — so it
cannot start on a machine without a CUDA Toolkit installation. Packaging it
would ship a Store app that fails in the Windows loader before `main` runs.

**Fail the job loudly if the CPU artifact is missing.** A glob that silently
matches the wrong variant is the exact failure this paragraph exists to prevent,
and it would not be caught by any test that only asserts "an msix was produced".

## 4. The version — the guard is the deliverable, not the substitution

RFC 045 §4.1 gives the mechanism. What matters is **why the guard is there**:
`packaging/windows/AppxManifest.xml` currently hardcodes `Version="0.41.1.0"`,
which was already two releases stale, and that is precisely what RFC 042 §4's
"derive at build time, store nothing new" rule existed to prevent. A convention
alone did not hold; an enforced one will.

- Set the committed value to `0.0.0.0`.
- CI **throws** if the committed value is anything else, then substitutes
  `<tag>.0` into the staging copy.
- **Do not write the derived version back into the repository.** RFC 034 Part F
  keeps one source of truth; a second recorded version reintroduces the drift.

**Part F's comparison must append or strip `.0` symmetrically** when it checks
the Appx manifest. That is a comparison change. It is not a second version.

**Prove the guard with a deliberate failing case.** A guard exercised only by
the happy path has not been tested — set a wrong version, watch it fail, record
the output. This is an acceptance criterion, not a nicety.

## 5. Verification is the deliverable, not the `.msix`

A package that builds proves almost nothing. RFC 042 §3 proved the whole
install-and-launch sequence runs unattended **in 39 seconds, first attempt** — so
there is no cost argument for skipping it.

Install it, launch it via `shell:AppsFolder\<PackageFamilyName>!App`, and show
these three, each as observed output:

- **RFC 032** — no `ffmpeg`/`ffprobe` anywhere inside the `.msix`.
- **RFC 041** — data locations resolve to the packaged app's own directories.
  **This is where I most expect a real finding.** Package identity redirects
  `AppData`, and RFC 041 moved arama off exe-relative paths on the assumption of
  ordinary platform dirs. If it is wrong anywhere, it is wrong here.
- **Task 037** — `Subsystem 00000002 (Windows GUI)` on the binary *inside the
  package*. Verified on the shipped `0.41.2` zips already; this closes the same
  hole in the Store channel.

## 6. Traps

- **`AllowAllTrustedApps` is unset on hosted runners.** `Add-AppxPackage` fails
  for policy reasons that read like a package defect. Enable it via
  `HKLM:\…\AppModelUnlock`.
- **Launching the raw `.exe` tests nothing while appearing to pass.** It runs
  without package identity, so every packaged-identity behaviour — including
  §5's RFC 041 check — silently reports the unpackaged answer. Use `AppsFolder`.
- **Glob the SDK `bin` directory for `makeappx`/`signtool`.** The runner had
  `10.0.26100.0`; a hardcoded build number is a time bomb.
- **The self-signed certificate never leaves CI.** It exists only to make the
  package installable for its own smoke test. **The package attached to the
  release is the unsigned one** — the Store re-signs, and attaching a
  test-signed package would be worse than attaching none.
- **`ProcessorArchitecture` is absent from `<Identity>`**, so `makeappx` will
  produce a `neutral` package for an x64-only payload. Set `x64`.

## 7. The four manifest divergences — report, do not quietly "fix"

RFC 045 §8 lists four ways `packaging/windows/AppxManifest.xml` differs from the
manifest RFC 042 proved. **A package built from the committed manifest is live
on the Store, so it evidently certified.** Treat them as questions your
install-and-launch run can answer, not as bugs to correct on the way past.

`MinVersion="10.0.17763.0"` is the one with a real consequence — RFC 042 §3
records that `uap10:TrustLevel="mediumIL"` with `rescap:runFullTrust` requires
`10.0.19041.0`. If that holds, the package claims support for builds it cannot
run on, which is a runtime failure and not a certification one.

**Change what you can justify from your own observed run, and say which of the
four you changed, which you left, and on what evidence.** "Made it match the
probe" is not a reason.

## 8. Non-change scope

- **Half B.** §1.
- **The six existing release assets and RFC 030's naming.** The `.msix` is a
  seventh asset. Nothing else moves.
- **The Store listing's metadata** — text, screenshots, description. Manual, and
  RFC 045 §9 keeps it that way.
- **Any application code.** This is packaging and CI. If a packaged-build check
  fails, that is a finding to report, not a licence to change `app/src`.
- **`Cargo.toml` versions.** Untouched.

## 9. Acceptance criteria

- `arama-<tag>.msix` is attached to the GitHub release as a seventh asset, built
  from the **CPU** variant.
- The committed `packaging/windows/AppxManifest.xml` reads `Version="0.0.0.0"`,
  and CI substitutes `<tag>.0`.
- The version guard **fails on a wrong committed value**, shown by a deliberate
  failing run.
- RFC 034 Part F compares Appx-manifest-to-tag with symmetric `.0` handling.
- The package installs and launches via `AppsFolder` on a clean runner.
- The three §5 checks are reported as observed output.
- Each of the four §7 divergences is reported as changed-with-evidence or
  left-with-reason.
- Gates clean. A check not run is recorded as not run.

## 10. Required deliverables

A review request under `.git-exclude/review-request/NNN-<slug>/` opening with an
`## Entry point` section giving the pinned commit, the exact `git show` command,
and plain paths to every file.

Include the workflow run URL, the install-and-launch output, the three contract
checks, the deliberate guard failure, and §7's four-way report.
