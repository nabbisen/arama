# RFC 041: Application data locations

**Status.** Implemented (0.40.0). Accepted by the project owner 2026-08-16.

*As built, one correction to this RFC's own §2:* it recommended abandoning the
cache and rebuilding, citing RFC 015. That precedent does not apply — RFC 015
abandoned v1 data because its **format** had changed and it was unusable, while
here the data is valid and merely in another folder. Abandoning it would have
forced a full re-index, arama's slowest operation, scaling with library size.
**All three kinds migrate; nothing is abandoned.**

*Verified on all three platforms*, which mattered: the verification failed on
macOS and Windows for two different reasons — a process-global `ARAMA_DATA_HOME`
leaking between parallel tests, and a `NATIVE_SMOKE_*` marker written to stderr
while the workflow grepped stdout. Neither was a defect in this change; both
were defects in checking it. Linux alone would have reported success.
**Tracks.** arama writes its data to **three different anchors**, one of which
follows the current working directory and one of which is unwritable when the
application is packaged. Consolidate to a single platform-correct location.
**Touches.** `env/src/dir.rs`, `app/src/core.rs`'s `ConfigManager` construction,
and a first-run migration. No UI, no AI, no discovery.

## Summary

| Data | Anchor today | Set by |
|---|---|---|
| `settings.json` | **current working directory** | `ConfigManager::new().at_current_dir()` |
| `.arama-local/` — CLIP/wav2vec2 models, `bin/` | executable's directory | `env::current_exe().parent()` |
| `.arama-cache/` — SQLite database, thumbnails | executable's directory | `env::current_exe().parent()` |

Three anchors, two of them wrong for different reasons. This RFC proposes one
platform-appropriate per-user location for all three.

## Why now — two independent problems

### 1. Settings follow where you launched from

`app/src/core.rs:100` and `:237` both call `.at_current_dir()`, explicitly
overriding `app-json-settings`' platform default.

So launching arama from a different directory reads different settings. A user
who runs it from a shortcut, from a terminal in a project folder, and from the
install directory gets three different configurations, silently. Nothing tells
them which one is in effect.

**This is a defect independent of any packaging question**, and it is why this
RFC is not folded into the Store work: it should be fixed whether or not arama
is ever published there.

It also already undermines the portability property the exe-relative layout is
presumably for — "unzip it and everything lives in one folder" is only true if
you also launch from that folder.

### 2. The executable's directory is unwritable when packaged

Windows installs a packaged application to
`C:\Program Files\WindowsApps\<package>\`, which is **read-only by design**.
arama would try to create `.arama-local/` and `.arama-cache/` there on first
run — the setup wizard would have nowhere to put the CLIP model it downloads,
and the cache could not be created.

**Whether Windows refuses the write or silently redirects it must be
established by running it, not reasoned about.** A redirect would be worse than
a refusal: several hundred megabytes of model data landing somewhere the user
never chose, with uninstall behaviour unknown.

This blocks [RFC 042](../proposed/042-windows-store-distribution.md) entirely.

## The tool is already a dependency

`app-json-settings` resolves the platform directory correctly — `%APPDATA%` on
Windows, `~/Library/…` on macOS, XDG elsewhere — and arama already depends on
it. The `.at_current_dir()` calls are an explicit opt-out of behaviour the
project already has.

For settings, this RFC is mostly *deleting* two method calls.

## Goals

- One anchor for all three kinds of data, chosen by the platform.
- A packaged build works — nothing is written into the install directory.
- Existing users do not silently lose their configuration.
- Which location is in use is discoverable by a user, not inferred.

## Non-goals

- Store packaging itself. That is RFC 042 and depends on this.
- The CPU/GPU binary question. Also RFC 042.
- Any change to what is stored, only where.
- A user-configurable data location. If wanted, it is separate work.

## Design questions this RFC must settle

### 1. One location, or one per kind?

Models, cache, and settings differ: settings are tiny and precious, the cache is
large and regenerable, models are large and re-downloadable but expensive to
fetch again.

Most platforms distinguish these — XDG has `XDG_CONFIG_HOME`, `XDG_CACHE_HOME`
and `XDG_DATA_HOME`; macOS has `Application Support` versus `Caches`. Windows
has `%APPDATA%` versus `%LOCALAPPDATA%`, where the second is the correct home
for large regenerable data because it does not roam.

**Recommendation: follow the platform's own distinction** — settings in config,
models in data, cache in cache. Putting a multi-hundred-megabyte model into a
roaming Windows profile would be a real misbehaviour, and putting a cache
somewhere a backup tool captures it is waste.

### 2. What happens to existing data?

Three answers, one per kind:

- **Settings — migrate.** Small, and losing a user's indexed root directory and
  threshold is a visible regression. Read the old location once if the new one
  is absent, write to the new one.
- **Cache — abandon and rebuild.** It is a cache. RFC 015 already set this
  precedent by retiring the v1 migration path rather than carrying it.
- **Models — decide deliberately.** Re-downloading hundreds of megabytes because
  the application moved its own folder is a poor experience on a metered
  connection. Moving them once is cheap but is a large filesystem operation on
  first run, with its own failure modes.

Whatever is chosen, **a failed migration must not be silent** — RFC 017's tier
model already governs this class.

### 3. Is any portable mode retained?

The owner's position, recorded: portability is **not essential**. Its one real
benefit is the deletion story — remove the folder and everything is gone, which
matters for a privacy tool with no uninstaller — and that is already weakened by
settings living in the CWD.

**Recommendation: no portable mode.** One behaviour is simpler than two, and
"which mode am I in" is a support question this project does not need. If the
deletion story matters, address it directly — a documented path and a way to
see it from the UI — rather than by scattering data next to the binary.

## Testing and verification

- **All three platforms.** This is platform-resolution code; Linux passing says
  little about Windows. RFC 038's native-smoke workflow already runs on
  `windows-latest` and `macos-latest` and is the natural home.
- **A packaged Windows run** — the specific question §2 raises. Whether it
  refuses or redirects is a result either way and should be recorded, not
  assumed.
- **Migration from each prior layout**, including the case where a user has data
  in *both* the old and new locations.
- **The CWD defect specifically**: launching from two different directories must
  yield the same settings afterwards.

## Acceptance criteria

- One anchor per data kind, platform-appropriate, with no `.at_current_dir()`
  remaining.
- Nothing is written to the executable's own directory.
- Existing settings survive; cache and model handling matches §2's decisions.
- A user can discover where their data lives without reading source.
- Verified on Windows, macOS and Linux, not inferred from one.

## Risks

- **Silent data loss.** A user who upgrades and finds their configuration reset
  has been harmed by this change. §2's migration is the guard, and its failure
  path must be visible.
- **Two locations coexisting.** A user who runs both an old and a new build will
  have data in both. Deciding which wins, and saying so, is part of the work.
- **Scope creep into a configurable location.** Fenced in Non-goals.

## Open questions

- Do the executable assets still ship as a self-contained directory once nothing
  is written into it? The layout contract is RFC 030's; nothing here forces a
  change, but the *reason* for that shape weakens.
- Should the UI show the resolved data location? It costs little and turns
  "where did my cache go" into a self-answering question.
