# RFC 019 — Startup fatal-boundary resilience

**Status.** Proposed
**Tracks.** Roadmap follow-up: define which startup failures should abort the
application versus recover into a usable shell with visible feedback.
**Touches.** `app/src/main.rs`, `app/src/lib.rs`, `app/src/core.rs`,
`app/src/core/settings.rs`, `crates/ui/main/src/core/views/gallery.rs`,
`crates/i18n/src/en.rs`, `crates/i18n/src/ja.rs`, `docs/src/users/faq.md`,
`docs/src/dev/workspace.md`, `CHANGELOG.md`, `rfcs/README.md`.

## Summary

RFC 017 made recoverable runtime and page-load errors visible. Several startup
paths still use panic-style or ignored-error behavior even though some failures
can recover into a usable application shell.

This RFC proposes a first startup fatal-boundary pass:

1. Classify startup failures into fatal shell startup, recoverable degraded
   startup, and developer invariant.
2. Return or log top-level `iced::Result` failures instead of silently ignoring
   them.
3. Replace remaining recoverable startup `expect()` paths with explicit
   fallback and startup toasts.
4. Keep genuinely impossible or allocation-only failures outside the user-error
   surface.

The goal is not to make every startup path recover. The goal is to make the
boundary explicit: if arama can show a truthful shell, it should do so with
visible feedback; if it cannot, the failure should be returned or printed
clearly at process start.

## Why

Recent resilience work improved setup, cache, settings, similarity, and
AI/video indexing behavior. Startup still has a few unclear boundaries:

- `main()` ignores the `iced::Result` returned by `arama::start()`.
- `setup_validate()` calls `local_dir().expect("failed to get local dir")`.
- `setup_validate()` ignores the result of `validate_dir(&local_dir)`.
- `Gallery::new().expect("failed to init gallery")` is panic-shaped even
  though the current gallery constructor is infallible in practice.
- The persisted root directory is scanned synchronously during `App::new()`;
  invalid or inaccessible roots should recover into a truthful empty Explorer
  state with a visible warning, not a misleading tree.

Some of these are small cleanups, but they should be designed together because
startup failure behavior is user-visible and easy to regress.

## Design

### Part A — Startup failure policy

Classify startup failures into three tiers:

| Tier | Meaning | Default behavior |
|------|---------|------------------|
| Fatal shell startup | iced/window/runtime cannot create a usable app shell | Return the `iced::Result`; `main()` reports the error |
| Recoverable degraded startup | The shell can open but some startup data or local setup is unavailable | Start with fallback state and show a startup toast |
| Developer invariant | Impossible internal state or allocation-only failure | Keep panic/expect only with a precise justification |

First-pass classification:

| Failure | Tier | Behavior |
|---------|------|----------|
| `iced::application(...).run()` failure | Fatal shell startup | Return from `start()` and report in `main()` |
| App local directory resolution failure | Recoverable degraded startup | Skip local setup validation, show startup error toast |
| Local setup directory validation failure | Recoverable degraded startup | Continue to setup wizard or fallback setup, show startup warning/error toast |
| Setup view initialization failure | Recoverable degraded startup | Existing setup fallback plus startup error toast |
| Settings load failure | Recoverable degraded startup | Existing default settings plus startup warning toast |
| Gallery state initialization failure | Developer invariant if constructor remains infallible; otherwise recoverable degraded startup | Prefer making constructor infallible or handling `Result` without `expect()` |
| Persisted root directory scan failure | Recoverable degraded startup | Use an empty/placeholder Explorer tree and show startup warning toast |
| Static extension allowlist construction failure | Developer diagnostic | Keep stderr/log fallback from RFC 017 unless it can make displayed data misleading |

### Part B — Top-level application result

Change process startup so `iced::Result` is not ignored:

```rust
fn main() -> iced::Result {
    arama::start()
}
```

or an equivalent shape that reports the error before exiting. The important
contract is that a fatal shell startup failure is not silently discarded.

### Part C — Startup notice aggregation

Keep the existing startup-toast pattern, but make it explicit and reusable
inside `App::new()`:

- create startup notices while loading setup, settings, local paths, and root
  directory state;
- convert them to toasts before returning `Self`;
- prefer concise user-facing copy over raw debug output;
- include paths only when useful and non-secret.

This does not require a new global diagnostics system. A small local helper is
enough if it reduces repeated toast ID boilerplate.

### Part D — Local setup validation

Replace `setup_validate()`'s panic-shaped local directory lookup with a
fallible helper:

```rust
fn setup_validation_notice() -> Option<StartupNotice>;
```

Equivalent shapes are acceptable. The helper should:

- attempt to resolve `local_dir()`;
- attempt to `validate_dir(&local_dir)` when the path exists;
- report failures as startup notices;
- avoid aborting `App::new()` unless a later design proves there is no usable
  shell without the local directory.

Setup/download flows already have their own recoverable UI errors, so this RFC
only covers the startup preflight.

### Part E — Gallery and root directory initialization

Prefer making `Gallery::new()` infallible if it only constructs empty in-memory
maps. If it remains fallible, remove the `expect()` call and recover with an
empty gallery plus startup error toast.

For the root directory scan:

- keep the persisted root directory value visible in the header/settings;
- if scanning fails or returns an unusable tree, show an empty Explorer state
  and a startup warning toast;
- do not trigger the cache pipeline for an invalid root;
- allow the user to pick another directory normally.

If `swdir` does not expose scan errors for the current call path, the
implementation should at least validate the configured root with existing
filesystem checks before scanning and treat validation failure as recoverable.

### Part F — Documentation

Update user and developer docs to say:

- startup may recover with defaults or empty state when local config, setup
  preflight, or configured root directory cannot be used;
- fatal startup is reserved for failures that prevent opening the app shell;
- top-level application startup errors are returned/reported rather than
  silently ignored.

## Touches in detail

### `app/src/main.rs` and `app/src/lib.rs`

Do not discard `iced::Result`. Keep `arama::start()` as the library entry point
and make the binary entry point return or report the result.

### `app/src/core.rs`

Replace startup `expect()` paths and ignored setup validation with structured
startup notices. Keep the existing settings/setup fallback behavior, but reduce
duplicated toast construction if practical.

### `app/src/core/settings.rs`

No functional change expected unless top-level iced settings become part of the
startup error boundary.

### `crates/ui/main/src/core/views/gallery.rs`

Make `Gallery::new()` infallible if possible, or require callers to handle its
error without panic.

### `crates/i18n/src/en.rs` and `crates/i18n/src/ja.rs`

Add localized strings for local setup preflight failure and invalid startup
root directory if implementation needs new user-visible copy.

### `docs/src/users/faq.md`

Document degraded startup behavior and what users can do when the configured
root directory or local setup path is unavailable.

### `docs/src/dev/workspace.md`

Document the startup failure tiers and relate them to RFC 017's recoverable
runtime error tiers.

### `CHANGELOG.md`

Record the behavior change under `[Unreleased]` during implementation.

### `rfcs/README.md`

Add RFC 019 to the Proposed table while under review, then move it to
Implemented when shipped.

## Non-goals

- No change to model download or artifact verification policy.
- No retry queue or persistent diagnostics log.
- No broad rewrite of app initialization into dependency injection.
- No release action. Release timing and RFC lifecycle movement remain
  owner-managed.
- No attempt to recover from an iced/window failure by starting a different UI.

## Risks

- Recovering too broadly could hide a real startup defect. Mitigation: only
  recover when the app can show a truthful shell, and keep developer-invariant
  failures explicit.
- Startup toasts can stack up. Mitigation: aggregate related startup notices
  where practical and keep copy short.
- Root directory validation could disagree with `swdir` behavior. Mitigation:
  test invalid/missing roots and keep the user path editable after fallback.
- Returning `iced::Result` from `main()` may change terminal output. Mitigation:
  use Rust's standard `Result` main behavior or an explicit concise `eprintln!`.

## Test plan

- Focused tests for any helper that classifies startup notices:
  - local directory resolution failure becomes recoverable startup notice;
  - local validation failure becomes recoverable startup notice;
  - invalid configured root skips cache startup and records a warning.
- If `Gallery::new()` becomes infallible, compile-time call-site cleanup is
  enough; if it remains fallible, add a caller/helper test for fallback.
- Manual smoke:
  - start with a missing configured root directory and verify the app shell
    opens with a visible warning and no cache run;
  - start normally and verify no startup warning appears.
- Workspace gates:
  - `cargo fmt --all --check`
  - `cargo check --workspace`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Open questions

1. Should `main()` simply return `iced::Result`, or should it print a custom
   one-line error before returning a non-zero exit?
2. Should startup notices share RFC 017's toast titles/copy style, or should
   startup have its own concise titles?
3. Should invalid configured root directories keep the header text as the
   invalid path, or immediately reset the session root to `"."`?
