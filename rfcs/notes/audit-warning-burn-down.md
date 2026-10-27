# Audit Warning Burn-down

This note records the July 2026 maintenance pass over `cargo audit` warnings.
It is not an RFC because no product behavior, architecture policy, or public
contract changed.

## Outcome

The pass resolved the warnings that had compatible patch releases available in
the current dependency graph:

- `anyhow` 1.0.102 -> 1.0.103
- `memmap2` 0.9.10 -> 0.9.11

After the initial lockfile update, `cargo audit` still exited successfully with
allowed warnings. These warnings are tracked below instead of being added as
broad new ignores.

After the RFC 022 image similarity implementation, the active audit surface is
down to three allowed warnings: `bincode` 2.0.1 through `localcache`, `paste`
1.0.15 through multiple transitive owners, and `ttf-parser` 0.25.1 through the
font/rendering stack.

After the RFC 024 image codec minimization implementation, the active image
dependency graph no longer reaches `ravif` or `rav1e`. `cargo audit` still
reports the same three allowed warning crates because `paste` remains reachable
through Candle/gemm, tokenizers, and Apple-target rendering paths.

After the RFC 027 audit-ledger refresh, the active audit surface is four
allowed warnings: `bincode` 2.0.1 through `localcache`, `paste` 1.0.15 through
multiple transitive owners, `rustybuzz` 0.20.1 through the SVG/font rendering
stack, and `ttf-parser` 0.25.1 through the font/rendering stack. `cargo audit`
exits successfully with these warnings under the current policy.

## Image Similarity Search Follow-up

RFC 022 replaced the localized `hnsw_rs` image similar-pairs candidate
generator with exact bounded pairwise search. This removed the `hnsw_rs` path
and the transitive `bincode` 1.3.3 audit warning. The image similar-pairs
dialog now asks for the top 50 image pairs globally instead of passing `50` as
an HNSW neighbor count.

The same lockfile refresh removed the previously stale `proc-macro-error2`
entry that had no active all-target workspace dependency path during the
maintenance pass.

## Cache Serialization Dependency Follow-up

RFC 023 selected retention for the implementation outcome. `localcache` 0.20.0
is still the latest published crate version observed in this pass, and both the
published crate metadata and the local upstream checkout keep `bincode` as an
unconditional dependency. Although `localcache` exposes `Codec::Json`, enabling
JSON in 0.20.0 would not remove the `bincode` audit warning by itself.

No cache payload codec, cache namespace, payload version, workspace patch, or
cache-engine replacement is included. Burning down the remaining `bincode` 2.0.1
warning still requires an upstream `localcache` release that can compile a
bincode-free cache path, or a separately reviewed cache-engine replacement.

## Dependency Modernization Follow-up

RFC 020's implementation updated first-party Candle dependencies from 0.10 to
0.11 and the non-Linux sidecar `zip` dependency from 4.6.1 to 8.6.0. `zip`
8.6.0 declares Rust 1.88, which stays below arama's Rust 1.90 workspace floor.

That modernization did not remove the remaining audit warnings.
`pt2safetensors` 0.1.3 still owns a transitive `candle-core` 0.10.2 path for
PyTorch to SafeTensors conversion, so Candle-related lockfile duplication is
not fully removed by the first-party Candle update.

## CLIP Source Strategy Follow-up

RFC 021 selected "retain runtime conversion" as the implementation outcome.
The currently pinned OpenAI CLIP revision publishes `pytorch_model.bin` and no
`model.safetensors` file in the inspected Hugging Face tree. Until arama has a
trustworthy pinned SafeTensors source or owner-managed mirror with provenance,
checksum, and embedding-regression evidence, `pt2safetensors` remains an
intentional dependency and the transitive Candle 0.10 path remains tracked.

## Image Codec Dependency Follow-up

RFC 024 switched `iced` from its codec-enabling `image` feature to
`image-without-codecs` and configured the workspace `image` dependency with
explicit codec features for arama's current accepted image formats: PNG, JPEG,
WebP, GIF, and BMP. The product image extension allowlist is unchanged.

`cargo tree -i ravif@0.13.0` and `cargo tree -i rav1e@0.8.1` now report no
active reverse dependency path. This removes the AVIF/ravif/rav1e owner path
from the `paste` warning, while leaving the remaining Candle/gemm, tokenizers,
and target-qualified rendering owners tracked below.

## Audit Ledger Refresh Follow-up

RFC 027 reconciled this note with the current `cargo audit` output. The refresh
does not change dependencies, add audit ignores, or introduce
`cargo audit --deny warnings`; it only records the current owner paths and
clarifies that allowed warnings still require rationale and revisit conditions.

## Event-listener Follow-up

`event-listener` 5.4.1 was flagged by `cargo audit` as RUSTSEC-2026-0221 (an
unsound `!Send` tag crossing thread boundaries via `StackSlot`), reaching
arama via `zbus` and the `async-*` desktop-portal stack behind `rfd` and
`file-handle` — not via `localcache`. This note never listed it: a Task 008
review request claimed all five then-current warnings were "already
tracked," and review 073 found that claim wrong, since this ledger has only
ever recorded the four below.

Task 010 Item A resolved it directly: `cargo update -p event-listener`
moved the locked graph to 5.4.2, published and unyanked, which does not
carry the advisory. `cargo audit`'s allowed-warning count went from five to
four as a result. No ignore, patch, or override was needed.

## Remaining Warnings

### `bincode` 2.0.1

Observed path:

```text
bincode 2.0.1 <- localcache 0.20.0 <- arama-cache
```

`localcache` 0.20.0 is the latest published crate version at the time of this
ledger refresh. `bincode` 3.0.0 exists, but arama reaches `bincode` through
`localcache`, not a direct workspace dependency. Burning down this warning
requires an upstream `localcache` release that moves off the unmaintained
`bincode` line, or a cache-engine design change.

### `paste` 1.0.15

Observed through multiple transitive owners, including:

- first-party `candle-core` 0.11 through `gemm`
- `pt2safetensors` 0.1.3 through transitive `candle-core` 0.10 and `gemm`
- `tokenizers` through `macro_rules_attribute`
- Apple-target rendering stack through `metal`, `wgpu-hal`, `wgpu`, and `iced`

This is not owned by a single direct dependency. Treat it as a dependency
modernization signal and revisit when direct dependency updates are planned.

### `rustybuzz` 0.20.1

Observed through the SVG/font rendering path:

```text
rustybuzz 0.20.1 <- usvg 0.45.1 <- resvg 0.45.1 <- iced_tiny_skia / iced_wgpu <- iced
```

This is target- and renderer-adjacent dependency risk rather than an arama
source-code dependency. Revisit when the iced rendering stack, `resvg`, or
`usvg` expose a compatible path away from the unmaintained `rustybuzz` line, or
when arama changes renderer/SVG support.

### `ttf-parser` 0.25.1

Observed through the font/rendering stack, including `fontdb`, `cosmic-text`,
`iced_wgpu`, `owned_ttf_parser`, `ab_glyph`, `usvg`, `resvg`, and `rustybuzz`
paths. `ttf-parser` 0.25.1 is the latest published crate version at the time of
this ledger refresh, so there is no compatible patch update to apply yet.

### `lru` 0.16.4 — added 2026-08-15

RUSTSEC-2026-0253, **unsound**: potential use-after-free from missing panic
safety in `LruCache::pop()`.

Observed path:

```text
lru 0.16.4 <- cryoglyph 0.1.0 <- iced_wgpu 0.14.0 <- iced_renderer 0.14.0 <- iced 0.14.0
```

Reached only through the iced rendering stack — no direct workspace dependency,
and no version of it that arama selects. Burning it down requires `cryoglyph`
or the `iced_wgpu` line to move to a patched `lru`. **Revisit when the iced
rendering stack next updates**, alongside `rustybuzz` and `ttf-parser`, which
enter through the same renderer and share that revisit condition.

Not previously listed because the advisory postdates this ledger's last
refresh; surfaced by the 0.39.1 release gate.

**Reclassified 2026-09-12, after snora's advisory-gate correction.** Three
things this entry had wrong or missing:

1. **This is not the same class as the other three.** `bincode`, `paste` and
   `ttf-parser` are *unmaintained-crate* advisories — nobody is maintaining the
   code. This is **memory corruption on a runtime path**: use-after-free /
   double-free. Listing it beside them, with the same treatment, understated it.
   snora now names it in their threat model rather than only in scanner
   configuration, and that distinction is the right one.
2. **`cargo update -p lru` will not move it.** `cryoglyph 0.1.0` holds `lru`
   below the patched **0.18.2** and has published no release that lifts it, so
   **neither arama nor snora can clear this today.** The previous wording
   ("burning it down requires `cryoglyph` or the `iced_wgpu` line to move") was
   right in direction but did not say that the obvious command is useless, which
   is the thing someone would try first.
3. **Reachability, which narrows it considerably.** The failure needs a stored
   key whose `Drop` panics, with unwinding enabled. arama stores no such key in
   any cache it owns; the `lru` instances here belong to `cryoglyph`'s glyph
   atlas. That is a real mitigation and it belongs on the record — but it is a
   mitigation, not an absence, and `panic = "abort"` is commented out in
   `Cargo.toml`'s release profile rather than set.

**And the shared revisit condition above was already shown to be wrong.** This
entry grouped `lru` with `rustybuzz` and `ttf-parser` as sharing one trigger.
snora 0.42.0 retired `rustybuzz` and moved neither of the others — recorded in
the Resolved section. Grouping advisories by how they entered predicts nothing
about how they leave.

### `quinn-proto` 0.11.15 — added 2026-10-21

RUSTSEC-2026-0338, 0339, 0340, 0341, 0342. Two (0340, 0341) are 7.5 high; the
other three are unscored. All five are QUIC protocol-level DoS/panic issues —
a malicious peer panicking `quinn-proto` with crafted STREAM/DATAGRAM frames,
or bypassing its memory-exhaustion guards.

**Not reachable at all, not merely mitigated.** `quinn`/`quinn-proto` enter
`Cargo.lock` because `reqwest` 0.13.4 declares them as the optional
dependencies of its own `http3` feature:

```toml
# reqwest 0.13.4's own Cargo.toml
http3 = ["rustls", "dep:h3", "dep:h3-quinn", "dep:quinn", "tokio/macros"]
```

arama's workspace `Cargo.toml` enables only `["blocking", "multipart",
"rustls", "stream"]` for `reqwest` — `http3` is not among them, and none of
those four imply it. Cargo still resolves `quinn`/`quinn-proto` into the lock
as a dormant optional-dependency entry because lock resolution covers the
full feature graph, not only the active feature set; it does not mean the
code is compiled or reachable.

Confirmed independently, not merely asserted from the manifest reading:

```sh
$ cargo tree -i quinn-proto
warning: nothing to print.
$ cargo tree -i quinn-proto --target all -e normal,build,dev
warning: nothing to print.
```

No edge, under any target, under any edge kind. **Revisit condition:**
`reqwest`'s `http3` feature actually appearing in arama's or arama-ai's
enabled feature set in `Cargo.toml` — a condition anyone can check by reading
that file, not "when upstream fixes it."

### `zip` 7.2.0 / 8.6.0 — added 2026-10-21

RUSTSEC-2026-0367 (SUID/SGID preservation during extraction), 0368 (symlink
escape outside the destination directory), 0369 (9.1 critical — applies Unix
permissions through symlinks without confining the target). Each matches
**both** `zip` versions in the lock, so three IDs cover six `cargo audit`
entries.

**Two different packages pull two different `zip` versions, and the
duplicate is ours:**

```
zip 7.2.0 <- candle-core 0.10.2 <- pt2safetensors 0.1.3 <- arama-ai
zip 8.6.0 <- candle-core 0.11.0 <- arama-ai
```

`pt2safetensors` 0.1.3 requires `candle-core ^0.10`; it is the only published
version (checked against the crates.io index, 2026-10-21) and cannot be
updated to track candle 0.11 — no newer release exists. **Not collapsed, and
not forced**: RFC 021 already documents this exact duplicate and the
condition for removing `pt2safetensors` (a direct SafeTensors source being
selected first); that condition still has not been met, and dropping it here
to shrink an advisory count would override a decision this ledger does not
get to override.

**The remaining edge is real and upstream-blocked.** `candle-core` 0.11.0 is
the latest published version and still depends on `zip` 8.6.0 (checked
against the crates.io index, 2026-10-21); `zip` fixed all three at `>=9.0.0`.
Revisit when `candle-core` depends on `zip >=9.0.0`.

**Whether arama's model path reaches the vulnerable function — determined,
not assumed.** All three advisories concern `ZipArchive::extract()` /
`ZipStreamReader::extract()` specifically (confirmed by reading each
advisory's own text, not the task's paraphrase of it): extraction to disk,
not reading an archive's entries. arama never calls `zip` directly —
`pt2safetensors` doesn't either (`grep -rn "zip\b" pt2safetensors-0.1.3/src`
finds nothing); the only consumer in this entire chain is `candle-core`
itself, reading PyTorch `.bin` checkpoints via its `pickle`/`npy` modules.
Checked both versions' source directly:

```sh
$ grep -rn "\.extract(" candle-core-0.11.0/src/ candle-core-0.10.2/src/
# (no matches, either version)
$ grep -n "zip::" candle-core-0.11.0/src/pickle.rs
677:    let mut zip = zip::ZipArchive::new(zip_reader)?;
689:        let reader = zip.by_name(file_name)?;
```

**Zero calls to `extract()` in either version.** Every use is `by_name()` /
`by_index()` — reading one entry's bytes into memory, never writing extracted
files to a destination directory. **The critical does not apply to arama's
actual usage, determined rather than guessed**: the only code that could call
the vulnerable function doesn't, in either version in the lock.

### `zvariant` 5.12.0 / `zbus_names` 4.3.2 / `zvariant_derive` 5.12.0 — added 2026-10-21

RUSTSEC-2026-0370, 0371, 0372 — all *unmaintained*, not vulnerabilities. Each
advisory (checked individually, not assumed identical) states the same thing:
merged into `zbus` as of `zbus` **6.0.0** (`zvariant` becomes zbus's `wire`
module, `zbus_names` becomes its `names` module, `zvariant_derive`'s macros
move into zbus directly). `zbus` 6.0.0 is already published.

Two real paths, both through dependencies this project doesn't control
directly:

```
zvariant <- zbus 5.16.0 <- file-handle 0.2.3 <- arama (direct) and arama-ui-widgets
zvariant <- zbus 5.16.0 <- mundy 0.2.3 <- iced_winit <- iced
```

**Checked whether either has already moved, rather than assuming not**:
`file-handle`'s latest published release is 0.4.0 (current lock: 0.2.3), but
it still requires `zbus ^5` (checked against its crates.io dependency
manifest, 2026-10-21) — the newer release did not adopt zbus 6. `mundy`
0.2.3 is still its only published version and requires `zbus ^5.0.0`. Neither
upgrade clears this today.

**Revisit condition:** `file-handle` and `mundy` both depending on `zbus
>=6.0.0` — a version number on each, checkable the same way this entry was
written, not "when upstream fixes it."

## Resolved: the tokio unsound advisory and the yanked clipboard-win — 2026-10-27

Two different kinds of problem, closed in the same change (Task 062) because
both are simple version moves with no ignore involved. Recorded separately so
a missing severity on one of them does not read as an oversight — this ledger
has never recorded a yanked crate before.

**RUSTSEC-2026-0386, `tokio` 1.52.3 — unsound, not a vulnerability.**
`AsyncFd::new` and its three sibling constructors (`with_interest`, `try_new`,
`try_with_interest`) violate I/O safety. Governed by RFC 049 §3's reachability
condition even though no ignore is added here — determined, not guessed:

```text
tokio 1.52.3 <- arama (direct, and via most of the async stack)
```

Neither arama's own code nor tokio's internals reach the affected
constructors. Checked both, not assumed from the absence of a direct call —
the same standard T059 applied to `zip`'s `extract()`:

```sh
$ grep -rn "AsyncFd::\(new\|with_interest\|try_new\|try_with_interest\)\b" \
    --include="*.rs" src/        # tokio 1.52.3 and 1.54.1, both checked
# matches only inside src/io/async_fd.rs itself (the definitions) and doc
# comments (///); zero call sites anywhere else in src/ in either version
```

Nothing in tokio's own `net`/`runtime`/`fs` modules routes through these
public constructors — they exist for callers building a custom FD wrapper, a
pattern arama does not use. **Record this regardless of the fix**, per the
task's instruction: if a future advisory names the same functions, this
answer does not need to be redone.

**Yanked, not advisoried: `clipboard-win` 5.4.1.** The crate was withdrawn
from crates.io — no severity, no affected-functions list, because a yank
isn't an advisory. Windows-only:

```text
clipboard-win 5.4.1 <- window_clipboard 0.5.1 <- iced_winit 0.14.0 <- iced
```

`cargo tree -i clipboard-win --target x86_64-unknown-linux-gnu` prints
nothing; only the `x86_64-pc-windows-msvc` target shows the chain above. No
Linux gate exercises this crate at all — the Windows CI job is the only one
that proves this change.

**Resolution, same shape as the `event-listener` entry above** — direct
update, nothing suppressed:

```sh
cargo update -p tokio --precise 1.54.1
cargo update -p clipboard-win --precise 5.4.3
```

| Package | before | after |
|---|---|---|
| `tokio` | 1.52.3 | **1.54.1** (1.54.0 is yanked — not landed on) |
| `clipboard-win` | 5.4.1 | **5.4.3** |

Exactly two `[[package]]` blocks changed their own `version`/`checksum` —
confirmed by diffing name/version/checksum lines in isolation, not the raw
`Cargo.lock` diff. The raw diff also shows eight dependency-array lines
(in `dirs-sys`, `errno`, `rustix`, `rustls-platform-verifier`, `socket2`,
`tempfile`, `uds_windows`, `winapi-util`) repointing from `windows-sys 0.61.2`
to an older sibling version. **Not a third moved package**: all four
`windows-sys` versions (0.52.0, 0.59.0, 0.60.2, 0.61.2) are present as
`[[package]]` entries both before and after this change, unchanged in
number — the resolver re-selected among already-present alternatives when
tokio's and clipboard-win's own requirement ranges shifted. Total
`[[package]]` block count: 705 before, 705 after.

`cargo audit --deny warnings` exits **0** after this change — the goal of the
task, and the first time since the database's last move that it does.

**Recorded because it is now a pattern, not an incident**: this is the
**second time in three days** `Audit` has gone red on a commit that changed
no code — T059 made it green on 2026-10-21 (11 remaining, all triaged), and
the advisory database moving under `c97e353` (a docs-only commit) broke it
again on 2026-10-24. RFC 049 exists because of this exact shape.

## Resolved: the rustls / rustls-webpki / webpki-root-certs advisories — 2026-10-21

Six advisories across three packages, all on **arama's only network path** —
the same path the RUSTSEC-2026-0285 fix below sits on:

```
rustls / rustls-webpki / webpki-root-certs <- reqwest 0.13.4 <- arama-ai
```

**RUSTSEC-2026-0382, `rustls-webpki` 0.103.15 — the most serious of the six.**
Certificate verification may accept distrusted intermediates.

**RUSTSEC-2026-0383, `webpki-root-certs` 1.0.8 — same class**, in the bundled
root set.

**RUSTSEC-2026-0346, 0347, 0348, 0349, `rustls` 0.23.45 — medium-to-low.** A
`Debug` impl exposing TLS secrets, an ECH rejection leak, a brotli
decompression allocation bound, and a downgrade.

**`--precise` required, the same mechanism the entry below found.** A plain
`cargo update -p rustls` stops short of the fixed version and reports
`(available: ...)` without taking it — a green local build at the intermediate
version proves nothing. What closes it:

```sh
cargo update -p rustls --precise 0.23.46
cargo update -p rustls-webpki --precise 0.103.16
cargo update -p webpki-root-certs --precise 1.0.10
```

which moves exactly three packages, confirmed by diffing `Cargo.lock`'s
`[[package]]` blocks rather than eyeballing:

| Package | before | after |
|---|---|---|
| `rustls` | 0.23.45 | **0.23.46** |
| `rustls-webpki` | 0.103.15 | **0.103.16** |
| `webpki-root-certs` | 1.0.8 | **1.0.10** |

`cargo audit` reports **11 vulnerabilities remaining** after this change, not
0 — the `zip`×6 and `quinn-proto`×5 groups, triaged separately (Task 059).
This entry closes only the six advisories named above.

## Resolved: the quick-xml ignores retired — 2026-10-02

**RUSTSEC-2026-0194 and RUSTSEC-2026-0195, `quick-xml` 0.39.4 — both
denial-of-service, both vulnerability ignores in `.cargo/audit.toml`, not
warnings.** Reached only through `wayland-scanner`, a build-time proc-macro
parsing protocol XML that ships with the build — neither was reachable at run
time by a user of arama, which is why this was low urgency despite being a
real, CVSS-scored vulnerability ignore.

**The condition that retires this ignore was met on 2026-07-22, not when this
task acted.** The ignore's own comment said to revisit "when wayland-scanner
publishes a version that accepts quick-xml 0.41 or newer, or when the Linux
Wayland dependency path changes." `wayland-scanner 0.31.11`, published
2026-07-22, does exactly that — the fix sat available, unnoticed, for over
two months.

```sh
cargo update -p wayland-scanner
```

moves exactly two packages, identical package set otherwise:

| Package | before | after |
|---|---|---|
| `wayland-scanner` | 0.31.10 | **0.31.11** |
| `quick-xml` | 0.39.4 | **0.41.0** — above the fixed line |

Both `RUSTSEC-2026-0194` and `RUSTSEC-2026-0195` removed from
`.cargo/audit.toml`'s `ignore` list in the same change, and the comment
explaining the now-gone constraint rewritten rather than left describing a
constraint that no longer exists. `cargo audit --deny warnings` passes clean
with both gone — the gate is stricter afterward, not looser: either ID
returning would now fail CI instead of being silently suppressed.

**This is the second revisit condition that passed unobserved** — the first
was `rustybuzz`, retired by snora 0.42.0's own dependency reduction and only
noticed while preparing Task 044 (recorded below). Whether a ledger entry's
own stated revisit condition deserves an actual automated check, rather than
depending on a future task happening to re-read the comment, is recorded as an
open question for the architect/owner in T049's own review package rather
than decided here — a plausible shape (a CI step that re-resolves each ignored
advisory's named blocking dependency and fails if a newer compatible version
exists) was not implemented, since it is a process/tooling decision beyond
this task's own scope of closing the two advisories.

## Resolved during a gate — 2026-10-02

**RUSTSEC-2026-0285, `rustls` 0.23.40 — vulnerability, not a warning.** TLS 1.3
handshake messages incorrectly accepted across encryption level boundaries,
severity 5.3 (medium).

Unlike every entry in **Remaining Warnings** above, this one sits on **arama's
only network path** — model downloads over HTTPS:

```
rustls 0.23.40 <- hyper-rustls 0.27.9 <- reqwest 0.13.4 <- arama-ai
```

**The obvious fix stops short.** `cargo update -p rustls` (with or without also
listing `aws-lc-rs`/`aws-lc-sys` on the same command) locks only as far as
0.23.43 and reports `(available: v0.23.45)` without taking it — still
vulnerable, and a green local build at 0.23.43 is not evidence of anything here.
**What closes it:**

```sh
cargo update -p rustls --precise 0.23.45
```

which moves four packages, the same package set before and after (705 crate
dependencies, unchanged):

| Package | before | after |
|---|---|---|
| `rustls` | 0.23.40 | **0.23.45** |
| `rustls-webpki` | 0.103.13 | 0.103.15 |
| `aws-lc-rs` | 1.17.0 | 1.18.1 |
| `aws-lc-sys` | 0.41.0 | 0.45.0 |

`aws-lc-sys` 0.45.0 gains one new dependency edge, to `pkg-config` — already
present elsewhere in the graph, not a new package entering it.

**Why plain `-p rustls` stops at 0.23.43 was not established.** Ruled out:
MSRV (all four moved packages declare `rust_version` 1.71, below arama's 1.91,
both before and after); the version being yanked (0.23.44 and 0.23.45 both
resolve cleanly under `--precise`); a third dependent constraining the graph
(`cargo tree -i aws-lc-rs`/`aws-lc-sys` shows `rustls` is their only path into
the tree); and avoiding a cascade into `aws-lc-rs`/`aws-lc-sys` specifically
(requesting all three together, still without `--precise`, still stops at
0.23.43, with `aws-lc-rs`/`aws-lc-sys` moving to 1.18.1/0.45.0 regardless).
The mechanism behind cargo's `update -p`-scoped version selection here remains
unexplained; recorded as such rather than guessed at.

`cargo audit --deny warnings` passes after the fix.

**The risk this carried**: `aws-lc-sys` compiles C (cmake/nasm on Windows) — a
four-minor jump there is exactly the class of lockfile change that broke the
Windows build for six commits in T048 (`rfcs/notes/windows-crate-reresolution.md`)
while every Linux gate stayed green. The Windows job in `Native FFmpeg Smoke`
was treated as mandatory evidence for this reason, not optional; a local
`cargo check --target x86_64-pc-windows-gnu` passing is reported only as a
supplementary signal — it does not exercise a native C build under MSVC.

## Resolved by the snora 0.42.0 upgrade — 2026-09-03

**RUSTSEC-2026-0206, `rustybuzz` 0.20.1 — resolved by removal, not by a patch.**

The entry below recorded that `rustybuzz` reached arama only through
`usvg <- resvg <- iced_tiny_skia / iced_wgpu <- iced`, and set the revisit
condition as *"when the iced rendering stack next updates, alongside `ttf-parser`,
which enters through the same renderer."*

**That condition was met by Task 047** (`f248676`, snora 0.39.1 → 0.42.0). snora
0.42.0 stopped enabling `iced`'s `svg` feature transitively, which dropped
`resvg` and `usvg` from the graph — and `rustybuzz` with them. **It is no longer
in `Cargo.lock` at all** (verified: zero occurrences), so there is nothing left
to track.

`cargo audit` accordingly reports **four** allowed warnings, not five:
`bincode`, `paste`, `ttf-parser`, `lru`.

**`ttf-parser` did *not* go with it**, despite sharing the revisit condition — it
still enters through `fontdb`, `cosmic-text`, `ab_glyph` and `owned_ttf_parser`,
which are font paths independent of SVG. The shared revisit condition was
correct as a trigger and wrong as a prediction that both would resolve together.

*Noticed 2026-09-03 while preparing Task 044, not during Task 047's own review —
the upgrade's dependency reduction was reported in that package as a binary-size
benefit, and its effect on the advisory ledger went unremarked by both sides.*

## Resolved during a release gate — 2026-08-15

**RUSTSEC-2026-0257, `webbrowser` 1.2.1 — vulnerability, not a warning.**
Unix `BROWSER` handling allowed browser argument injection.

Unlike every entry above, this one was **directly actionable**: `webbrowser` is
a first-party dependency of `arama-ui-widgets`, the workspace pin is
`webbrowser = "1"`, and the fixed line (`>= 1.2.2`) is semver-compatible. Closed
by `cargo update -p webbrowser` (1.2.1 → 1.2.4) with no manifest change and no
other package moving in the lockfile.

Recorded here rather than only in the changelog because it is the first time
this project's release gate has caught a **vulnerability** rather than a
warning, and because the reason it was catchable — a direct dependency on a
permissive pin — is the distinction that separates it from the four entries
above that arama cannot act on alone.

## Existing Explicit Ignore

`.cargo/audit.toml` still has only the scoped `quick-xml` ignores added during
release-gate recovery. Those advisories enter through `wayland-scanner 0.31.10`;
the fixed `quick-xml` line requires a newer range than the current Wayland
scanner constraint accepts.
