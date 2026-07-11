# RFC 017 — Visible recoverable error UX

**Status.** Proposed
**Tracks.** Roadmap follow-up: define a consistent policy for recoverable
application and UI load failures that should be visible to users instead of
silently falling back to defaults or stderr-only diagnostics.
**Touches.** `app/src/core.rs`, `app/src/core/message.rs`,
`app/src/core/update/*`, `crates/ui/main/src/core/views/cache_page*`,
`crates/i18n/src/en.rs`, `crates/i18n/src/ja.rs`, `docs/src/users/*.md`,
`docs/src/dev/workspace.md`, `CHANGELOG.md`, `rfcs/README.md`.

## Summary

Recent resilience work removed many panic paths and added toasts for several
cache, setup, and similarity failures. Some remaining recoverable failures are
still only printed to stderr or silently replaced with default state.

This RFC defines a first user-visible recoverable error pass:

1. Classify recoverable failures by user impact.
2. Use the existing toast channel for transient app-level failures.
3. Add inline Cache page load-error state for cache-table/footprint reload
   failures.
4. Surface settings load/save failures without crashing.

The goal is not to make every internal error visible. The goal is to make
actionable user-facing failures visible, predictable, and testable.

## Why

The current UI already has a toast system and several recovered errors use it,
but behavior is inconsistent:

- `App::new()` falls back to default settings if settings cannot be loaded, but
  currently only logs that failure to stderr.
- `App::save_settings()` still uses `expect("failed to save config")`, so a
  recoverable save failure can crash during ordinary UI changes.
- `CachePage::load_task()` converts load errors to an empty/default page and
  logs the failure to stderr. The user sees an empty state that may be false.
- Some extension allowlist errors fall back to unfiltered scans and only log to
  stderr. That may be acceptable because the allowlist is static application
  data, but the policy should say why.

These failures are not equivalent. A corrupt settings file, failed settings
write, and failed Cache page reload have direct user impact. An impossible
static extension allowlist should remain a developer diagnostic unless it can
lead to misleading user data.

## Design

### Part A — Error visibility policy

Classify recoverable failures into four display tiers:

| Tier | Meaning | Default UI |
|------|---------|------------|
| Fatal startup | App cannot initialize a usable shell | Return/startup failure |
| Blocking view | A page cannot truthfully show requested data | Inline error + retry |
| Recoverable action | A user action failed but the app remains usable | Error toast |
| Developer diagnostic | Static/invariant failure recovered with safe behavior | stderr/log only |

First-pass classification:

| Failure | Tier | Behavior |
|---------|------|----------|
| Setup initialization failure | Recoverable action | Existing startup error toast and fallback setup |
| Settings load failure | Recoverable action | Startup warning/error toast, use defaults |
| Settings save failure | Recoverable action | Error toast, keep in-memory state |
| Cache page rows/footprint load failure | Blocking view | Inline Cache page error + retry, keep stale rows when available |
| Cache clear/prune failure | Recoverable action | Existing app-level error toast |
| Thumbnail cache/embedding errors | Recoverable action | Existing app-level error toast |
| Static extension allowlist construction failure | Developer diagnostic | stderr/log and safe fallback |

### Part B — App-level settings errors

Replace the settings-save panic path with a fallible helper:

```rust
fn save_settings(&mut self) -> bool;
```

or equivalent. The helper should:

- attempt to save the current `Settings`;
- push an error toast if saving fails;
- return whether the write succeeded when callers need to know;
- keep the current in-memory state even when persistence fails.

`App::new()` should push a startup toast when settings cannot be loaded and
defaults are used. The toast text should make clear that the app is running
with default settings for this session.

### Part C — Cache page load error state

Change Cache page loading so errors remain structured:

```rust
pub struct CacheLoadError {
    pub message: String,
}

pub enum Internal {
    RowsLoaded(Result<CacheLoad, CacheLoadError>),
    // ...
}
```

Equivalent shapes are acceptable, but the key contract is:

- `load_task()` must not turn load failure into a successful empty result.
- `CachePage` stores `load_error: Option<String>` or equivalent.
- On load success, clear the stored error and update rows/footprint.
- On load failure, stop the busy indicator and show an inline error.
- If stale rows already exist, keep them visible and mark the reload failure.
- The Cache page should expose a retry/refresh action using the existing
  refresh control rather than inventing a second command.

The app may also push a toast for Cache page load failures, but the inline state
is the authoritative page-level feedback because the page data itself is
unavailable or stale.

### Part D — Copy and localization

Add localized strings for:

- settings load fallback;
- settings save failure;
- Cache page load failure;
- Cache page stale-data/retry copy if needed.

Avoid exposing raw debug formatting where a normal error string is enough.
Include the failing path when it is useful and non-secret, such as the config
file path if available from the settings library. Do not print or infer secrets.

### Part E — Documentation

Update user docs to say:

- settings load failures fall back to defaults and show a notification;
- settings save failures leave the current session state active but may not
  persist across restart;
- Cache page load failures are shown inline and can be retried.

Update developer docs with the display-tier policy so future error handling
work has a default decision rule.

## Touches in detail

### `app/src/core.rs`

Surface settings load fallback through startup toasts. Replace
`save_settings(&self)` with a fallible/toasting path that does not panic.

### `app/src/core/update/*`

Update callers that persist settings after user actions. Callers should keep
current in-memory state and let the toast explain persistence failure.

### `crates/ui/main/src/core/views/cache_page*`

Carry `Result<CacheLoad, CacheLoadError>` through the page update path and
render inline load failure state without clearing truthful stale data.

### `crates/i18n/src/en.rs` and `crates/i18n/src/ja.rs`

Add strings for the new user-visible error copy.

### `docs/src/users/`

Document settings and Cache page recoverable error behavior where users already
learn about settings/cache management.

### `docs/src/dev/workspace.md`

Document the display-tier policy and point future work at this RFC.

### `CHANGELOG.md`

Record the behavior change under `[Unreleased]`.

### `rfcs/README.md`

Add RFC 017 to the Proposed table while under review, then move it to
Implemented when shipped.

## Non-goals

- No broad AI/video retry policy. Video pipeline resilience remains a separate
  roadmap candidate because it needs skip/retry/abort semantics.
- No replacement of the toast system.
- No persistent error log or diagnostics export.
- No release action. Release timing remains owner-driven.
- No attempt to make developer-invariant failures noisy to users when the
  fallback is truthful and safe.

## Risks

- Too many toasts can become noise. Mitigation: reserve toasts for user-impact
  events and keep page-data failures inline.
- Keeping stale Cache page rows after a failed reload could be mistaken for
  fresh data. Mitigation: pair stale rows with visible reload-failed copy.
- Settings save failure could leave session state different from next-start
  state. Mitigation: toast clearly says persistence failed.
- Localization can lag behavior. Mitigation: add English and Japanese strings
  in the implementation batch.

## Test plan

- Unit tests for Cache page load success/failure update behavior:
  - success clears `load_error`;
  - failure sets `load_error`, clears `busy`, and preserves stale rows;
  - retry uses the existing refresh path.
- App-level tests or focused helper tests for settings save failure if the
  settings library can be injected or isolated.
- Existing cache/setup/similarity tests remain green.
- Workspace gates:
  - `cargo fmt --all --check`
  - `cargo check --workspace`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Open questions

1. Should Cache page load failures also produce a toast, or should inline state
   be the only notification?
2. Should settings load fallback be a warning toast or an error toast?
3. Is `save_settings(&mut self) -> bool` enough, or should the app introduce a
   small reusable `AppNotice`/`UiError` helper before more workflows adopt the
   policy?
