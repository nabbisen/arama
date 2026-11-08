> **AUTHORIZED 2026-11-09 — owner:** *"Both accepted: `PRIVACY.md` and T071."*
> Written 2026-11-09 by the architect.
>
> **Independent of T061, T063, T064 and T070** — it touches neither
> `app/src/core/view.rs` nor the cache. It can run beside any of them.

# T071 Handoff — model downloads refuse plain HTTP, redirects included

Constrained by [RFC 021](../done/021-clip-safetensors-source-strategy.md):
*"existing first-run checksum-failure behavior must remain unchanged."*

## 1. Purpose

arama's only network traffic is the one-time model download. Make every request
it sends **HTTPS-only, including every redirect hop**, through **one** client
constructor, so the property cannot drift between two call sites.

## 2. Background

**Found 2026-11-09, prompted by another project's incident.** HuggingFace now
redirects model downloads to a regional server (`huggingface.co` → `302` →
`us.aws.cdn.hf.co`). A project that allowed downloads only from a fixed list of
hosts broke for every user. **arama did not**: it keeps no host list, follows
redirects, and trusts the bytes only after they match a SHA-256 pinned in source,
under a size bound. The architect confirmed it with a real 378 MB download
through `download_authenticated_file` on 2026-11-09 — redirected to the regional
server, checksum passed.

**That design is right and this task keeps it.** What it lacks is a scheme rule.
Both clients are built with reqwest's defaults:

| Site | Purpose | Built as |
|---|---|---|
| `crates/ai/src/model/model_container/lifecycle.rs` (`reqwest::Client::builder()`, ~line 256) | the model download | connect timeout 10 s, total 30 min |
| `crates/ui/main/src/core/components/setup/downloader/update.rs` (`metadata_client`, ~line 192) | the setup screen's `HEAD` for the size shown before download | `METADATA_TIMEOUT` |

Neither sets `https_only`. **A redirect to `http://` would be followed.** The
pinned checksum still refuses a wrong file, so this is not an integrity hole.
What a plain-HTTP hop allows is an on-path observer seeing *which* model is
fetched, and an on-path attacker breaking the download (or, for the `HEAD`,
showing a wrong size). Neither is acceptable for the one connection arama
promises to make.

**reqwest 0.13.4 enforces `https_only` on redirects too** — the architect read
it: `src/redirect.rs:324`, `if self.https_only && next_url.scheme() != "https"`
returns `url_bad_scheme`, wired from `ClientBuilder::https_only` at
`src/async_impl/client.rs:1015-1019`. **Confirm against the version in
`Cargo.lock`**, and cite the lines you read.

## 3. Required implementation

1. **One constructor.** Add a single public function in `arama-ai` (beside the
   model container — the crate that owns the download) that returns a
   `reqwest::ClientBuilder` with `https_only(true)` already set. Name it for
   what it is, e.g. `model_download_client_builder()`. Callers add their own
   timeouts.
2. **Both sites use it.** `lifecycle.rs`'s download client and
   `metadata_client` in `crates/ui/main/src/core/components/setup/downloader/update.rs`
   (`arama-ui-main` already depends on `arama-ai`). **No other
   `reqwest::Client::builder()` may remain in non-test code** — grep and show it.
3. **Errors stay readable.** A refused scheme surfaces through the existing
   download-error path. Do not add a new user-facing string unless the
   existing one would describe the refusal wrongly; if you add one, add it in
   both locales.

## 4. Change scope

- `crates/ai/src/model/model_container/lifecycle.rs`
- the module in `crates/ai/src/` that holds the new constructor (state which)
- `crates/ui/main/src/core/components/setup/downloader/update.rs`
- tests beside them
- `docs/src/users/first-run.md` (§6)

## 5. Non-change scope

- **No host allow-list.** That is the design that broke the other project.
  Integrity comes from the pinned checksum; this task adds a scheme rule only.
- The pinned URLs, revisions, SHA-256 digests, size bounds and timeouts.
- `download_authenticated_file`'s signature — it already takes a client.
- `PRIVACY.md` — the architect corrects it separately.
- `CHANGELOG.md` and every `Cargo.toml` `version` field.

## 6. Documentation updates

`docs/src/users/first-run.md`: one sentence beside the existing checksum
sentence — downloads use HTTPS only, including any redirect, and every file is
checked against its pinned checksum wherever it was served from.

## 7. Required tests

- **A client from the constructor refuses an `http://` URL without connecting:**
  bind a loopback listener (the pattern in
  `crates/ai/src/model/model_container/tests.rs` already does), send a request
  to `http://127.0.0.1:<port>/`, assert an error, **and assert the listener
  accepted zero connections**. The second assertion is the one that proves the
  rule fires before the network, not after.
- **The existing loopback download tests keep passing.** They build their own
  plain-HTTP client and pass it to `download_authenticated_file`; that is
  correct for a fixture and must not be changed to weaken the new rule.
- **The redirect half:** no local test can serve an `https` → `http` redirect
  without a TLS fixture. **Do not build one for this.** Cite the reqwest source
  lines (§2) instead, and state in the package that the redirect property rests
  on reqwest's implementation at the locked version.
- Gates: `cargo build`, `cargo test`, `cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`, all `--workspace --locked`;
  `cargo audit --deny warnings`.

## 8. Acceptance criteria

- Exactly one non-test construction site for a `reqwest` client, setting
  `https_only(true)`; both former sites use it.
- The zero-connections test passes.
- The reqwest redirect enforcement is cited from the locked version's source.
- One real download through arama's path still succeeds and passes its checksum
  — run once, by hand, and report the elapsed time and byte count. Delete the
  file afterwards.
- `docs/src/users/first-run.md` updated.
- All gates green.

## 9. Prohibited shortcuts

- A host allow-list, or any check on the redirect target's host name.
- A custom redirect policy that duplicates what `https_only` already does.
- Weakening or skipping the checksum or the size bound anywhere.
- Building a TLS fixture to test the redirect half.

## 10. Module boundaries

- The constructor lives in `arama-ai`; `arama-ui-main` calls it. No new crate,
  no new dependency, no lockfile change — if one appears, stop and report.

## 11. Compatibility constraints

- **No change for any user today:** both pinned URLs are `https`, and
  HuggingFace's current redirects are `https` (checked 2026-11-09).
- RFC 021's checksum-failure behaviour unchanged.

## 12. Security constraints

- The pinned SHA-256 remains the integrity guarantee. `https_only` adds
  confidentiality of *which* model is fetched and resistance to an on-path
  attacker breaking or falsifying the size shown; it replaces nothing.
- No new network surface, no new host, no telemetry.

## 13. Known risks

- **HuggingFace serving a redirect over plain HTTP** would now fail the download
  instead of being followed. That is the intended outcome; the error must say
  the download failed, not that the user has no connection.
- **The redirect property depends on reqwest.** A future reqwest that changed
  `https_only`'s redirect handling would silently weaken it. The citation in
  the package is what a later upgrade's reviewer checks.

## 14. Required evidence

- The grep showing one construction site.
- The zero-connections test output.
- The reqwest source lines read, with the locked version.
- The real-download run: elapsed time, bytes, checksum result.
- Each gate's output, observed.

## 15. Review-request format

A package under `.git-exclude/review-request/` (working copy, not in the
repository) with a `## Entry point` section: the commit, the `git show`
command, and full repo-relative paths. Report observed output; a check not run
is recorded as not run. You cannot push: report the commit, and the architect
pushes and watches every workflow on that SHA.

## 16. Handoff checklist

The 17 items the architect's operating instructions require of every Developer
Handoff, and where each is covered here.

| Required item | Covered in |
|---|---|
| Task title | Heading |
| Purpose | §1 |
| Background | §2 |
| Applicable RFC and requirements | Companion line (RFC 021) |
| Change scope | §4 |
| Explicit non-change scope | §5 |
| Required implementation | §3 |
| Required tests | §7 |
| Required documentation updates | §6 |
| Acceptance criteria | §8 |
| Prohibited shortcuts | §9 |
| Relevant code or module boundaries | §10 |
| Compatibility constraints | §11 |
| Security constraints | §12 |
| Known risks | §13 |
| Required evidence | §14 |
| Required review-request format | §15 |
