use std::{
    env,
    ffi::OsString,
    fs::create_dir_all,
    io::{Error, ErrorKind, Result},
    path::{Path, PathBuf},
};

use directories::ProjectDirs;

const LOCAL_DIR: &str = ".arama-local";
const BIN_DIR: &str = "bin";
const CACHE_DIR: &str = ".arama-cache";

/// RFC 041: overrides every data location (settings, models, cache) to
/// live under one directory instead of the platform default. The
/// replacement for the CWD/exe-relative isolation trick prior scratch
/// profiles relied on (RFC 036/040) — set once, no need to launch from a
/// particular working directory or collocate the binary with the profile.
///
/// Settings land at `$ARAMA_DATA_HOME/settings.json` directly (via
/// `ConfigManager::with_root_dir`, wired in `app/src/core.rs`); this
/// module answers only the models/cache half.
pub const DATA_HOME_ENV_VAR: &str = "ARAMA_DATA_HOME";

/// Resolves the `ARAMA_DATA_HOME` override from an environment lookup
/// function.
///
/// Task 023: a pure, testable seam, mirroring `app-json-settings`'s own
/// `config_dir_from(getenv)` (`app_json_settings::core::dir`) — the
/// public-facing [`data_home_override`] supplies the real environment via
/// `std::env::var_os`; tests supply a stub closure instead of mutating a
/// process-global environment variable, which is `unsafe` in this edition
/// and races Rust's parallel-by-default test harness (this is exactly the
/// bug Task 023 fixes: `ARAMA_DATA_HOME`, mutated by one test, was leaking
/// into every other test's concurrently-running call to `local_dir()`/
/// `cache_dir()`).
fn data_home_override_from(getenv: impl Fn(&str) -> Option<OsString>) -> Option<PathBuf> {
    getenv(DATA_HOME_ENV_VAR).map(PathBuf::from)
}

fn data_home_override() -> Option<PathBuf> {
    data_home_override_from(|name| env::var_os(name))
}

fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("", "", "arama").ok_or_else(|| {
        Error::other("could not resolve a platform data directory (no home directory found)")
    })
}

/// Where CLIP/wav2vec2 models live, given an already-resolved override (or
/// none). Pure - the other half of the same seam as
/// [`data_home_override_from`]: [`local_dir`] supplies the real override
/// via [`data_home_override`] at its one call site; tests supply an
/// explicit value directly, without touching `ARAMA_DATA_HOME` at all.
///
/// Task 029: this directory's own `bin/` subfolder (see [`local_bin_dir`])
/// is *not* the legacy ffmpeg location - that is always exe-relative,
/// [`legacy_local_dir`], regardless of this function's override. The two
/// were conflated here before RFC 041 (when `local_dir` was itself
/// exe-relative, so they were the same path); they no longer are.
fn local_dir_with_override(data_home: Option<&Path>) -> Result<PathBuf> {
    if let Some(root) = data_home {
        return Ok(root.join(LOCAL_DIR));
    }
    Ok(project_dirs()?.data_local_dir().to_path_buf())
}

/// Platform data-local directory (RFC 041): `%LOCALAPPDATA%\arama\data` on
/// Windows, `~/Library/Application Support/arama` on macOS, `$XDG_DATA_HOME`
/// (or `~/.local/share`)`/arama` on Linux — never the executable's own
/// directory, which is unwritable once packaged, and never the roaming
/// profile, which large model downloads should not synchronise into.
pub fn local_dir() -> Result<PathBuf> {
    local_dir_with_override(data_home_override().as_deref())
}

/// The current models directory's own `bin/` subfolder - kept out of
/// automatic ffmpeg discovery on the same reasoning as the true legacy
/// location ([`legacy_local_bin_dir`]), not because anything is known to
/// exist here. Nothing has ever written a managed ffmpeg pair to this
/// path; it did not exist before RFC 041.
pub fn local_bin_dir() -> Result<PathBuf> {
    Ok(local_dir()?.join(BIN_DIR))
}

/// Where the thumbnail/embedding cache lives, given an already-resolved
/// override (or none). See [`local_dir_with_override`] - the same seam.
fn cache_dir_with_override(data_home: Option<&Path>) -> Result<PathBuf> {
    if let Some(root) = data_home {
        return Ok(root.join(CACHE_DIR));
    }
    Ok(project_dirs()?.cache_dir().to_path_buf())
}

/// Platform cache directory (RFC 041): `%LOCALAPPDATA%\arama\cache` on
/// Windows, `~/Library/Caches/arama` on macOS, `$XDG_CACHE_HOME` (or
/// `~/.cache`)`/arama` on Linux.
pub fn cache_dir() -> Result<PathBuf> {
    cache_dir_with_override(data_home_override().as_deref())
}

/// RFC 041 migration: the pre-041 location for models/`bin/`, always
/// relative to the running executable regardless of platform or packaging.
/// Read-only from a migration's point of view — never written to.
pub fn legacy_local_dir() -> Result<PathBuf> {
    let current_exe = env::current_exe()?;
    let path = current_exe
        .parent()
        .expect("failed to get exe parent directory")
        .join(LOCAL_DIR);
    Ok(path.to_path_buf())
}

/// Task 029: the true pre-0.40.0 managed-ffmpeg location, where a
/// pre-RFC-032 install could actually have downloaded a pair -
/// `legacy_local_dir()/bin`, always exe-relative regardless of
/// `ARAMA_DATA_HOME`. RFC 032's ffmpeg discovery excludes this location
/// from automatic candidates and rejects its explicit selection; it must
/// resolve here, not to [`local_bin_dir`]'s current-data-directory path,
/// or the exclusion silently stops covering the directory it exists to
/// cover.
pub fn legacy_local_bin_dir() -> Result<PathBuf> {
    Ok(legacy_local_dir()?.join(BIN_DIR))
}

/// RFC 041 migration: the pre-041 location for the cache, always relative
/// to the running executable. Read-only from a migration's point of view.
pub fn legacy_cache_dir() -> Result<PathBuf> {
    let current_exe = env::current_exe()?;
    let path = current_exe
        .parent()
        .expect("failed to get exe parent directory")
        .join(CACHE_DIR);
    Ok(path.to_path_buf())
}

pub fn validate_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        return create_dir_all(path);
    }

    if !path.is_dir() {
        return Err(Error::new(
            ErrorKind::NotADirectory,
            format!(
                "Can't treat cache directory, bacause invalid file is found: {}",
                path.to_string_lossy(),
            )
            .as_str(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 041 §5: "the mistake most likely to pass review and hurt users
    /// later." `local_dir()`/`cache_dir()` only compute a path - unlike
    /// `validate_dir`, they never touch the filesystem - so this is safe
    /// to run on a real machine, CI or not, without any override.
    #[test]
    #[cfg(windows)]
    fn windows_local_and_cache_dirs_resolve_under_the_local_not_roaming_profile() {
        let local = local_dir().unwrap().to_string_lossy().to_lowercase();
        let cache = cache_dir().unwrap().to_string_lossy().to_lowercase();
        assert!(
            local.contains("local") && !local.contains("roaming"),
            "models must resolve under %LOCALAPPDATA%, not the roaming profile: {local}"
        );
        assert!(
            cache.contains("local") && !cache.contains("roaming"),
            "cache must resolve under %LOCALAPPDATA%, not the roaming profile: {cache}"
        );
    }

    /// Same read-only guarantee as the Windows check above, for macOS's own
    /// split: `directories` has no roaming/local distinction there, but
    /// models and cache must still land in the two conventional, distinct
    /// per-user locations rather than collapsing onto the same folder.
    #[test]
    #[cfg(target_os = "macos")]
    fn macos_local_and_cache_dirs_resolve_under_their_conventional_locations() {
        let local = local_dir().unwrap();
        let cache = cache_dir().unwrap();
        assert!(
            local.to_string_lossy().contains("Application Support"),
            "models must resolve under ~/Library/Application Support: {local:?}"
        );
        assert!(
            cache.to_string_lossy().contains("Library/Caches"),
            "cache must resolve under ~/Library/Caches: {cache:?}"
        );
        assert_ne!(local, cache);
    }

    /// Linux's split is driven by `$XDG_DATA_HOME`/`$XDG_CACHE_HOME`
    /// (falling back to `~/.local/share`/`~/.cache`). Overriding those env
    /// vars, unlike `ARAMA_DATA_HOME`, exercises the real `directories`
    /// resolution path rather than bypassing it - still zero filesystem
    /// I/O, so still safe anywhere.
    ///
    /// Task 023: this is the one test in this file still allowed to mutate
    /// real process environment, and it is safe only because nothing else
    /// in *this* binary shares that assumption - it is the sole test that
    /// still calls the real, env-reading `local_dir()`/`cache_dir()` (every
    /// other test below goes through the `_with_override`/`_from` pure
    /// seam instead, precisely to avoid this), and `#[cfg(target_os)]`
    /// makes it mutually exclusive with the Windows/macOS platform checks
    /// above, which never coexist with it in one compiled binary. Do not
    /// add a second real-`local_dir()`/`cache_dir()`-calling test to this
    /// file without re-establishing that isolation.
    #[test]
    #[cfg(target_os = "linux")]
    fn linux_local_and_cache_dirs_respect_xdg_overrides() {
        let previous_data = env::var_os("XDG_DATA_HOME");
        let previous_cache = env::var_os("XDG_CACHE_HOME");
        let scratch = std::env::temp_dir().join(format!(
            "arama-dir-test-{}-xdg-override",
            std::process::id()
        ));
        unsafe {
            env::set_var("XDG_DATA_HOME", scratch.join("data"));
            env::set_var("XDG_CACHE_HOME", scratch.join("cache"));
        }

        let local = local_dir().unwrap();
        let cache = cache_dir().unwrap();

        unsafe {
            match &previous_data {
                Some(value) => env::set_var("XDG_DATA_HOME", value),
                None => env::remove_var("XDG_DATA_HOME"),
            }
            match &previous_cache {
                Some(value) => env::set_var("XDG_CACHE_HOME", value),
                None => env::remove_var("XDG_CACHE_HOME"),
            }
        }

        assert_eq!(local, scratch.join("data/arama"));
        assert_eq!(cache, scratch.join("cache/arama"));
    }

    /// Task 023 (Defect 1): `data_home_override_from`'s glue reads the
    /// right variable name and turns a present value into a path - a stub
    /// closure proves this without ever touching real process environment,
    /// so this test cannot race any other test no matter how it is run.
    #[test]
    fn data_home_override_from_reads_the_named_var_via_the_injected_getenv() {
        let seen = std::cell::RefCell::new(None);
        let result = data_home_override_from(|name| {
            *seen.borrow_mut() = Some(name.to_owned());
            Some(OsString::from("/scratch/arama-data-home"))
        });

        assert_eq!(seen.into_inner().as_deref(), Some(DATA_HOME_ENV_VAR));
        assert_eq!(result, Some(PathBuf::from("/scratch/arama-data-home")));
    }

    #[test]
    fn data_home_override_from_is_none_when_the_injected_getenv_finds_nothing() {
        assert_eq!(data_home_override_from(|_| None), None);
    }

    /// The override must affect both locations identically, or a scratch
    /// profile using it would see models and cache split across two
    /// different roots - the exact "both locations populated" trap RFC 041
    /// exists to avoid, self-inflicted by the test isolation mechanism.
    ///
    /// Task 023 (Defect 1): previously proved this by mutating
    /// `ARAMA_DATA_HOME` and calling the real `local_dir()`/`cache_dir()` -
    /// exactly the process-global mutation that raced
    /// `macos_local_and_cache_dirs_resolve_under_their_conventional_locations`
    /// on CI. Calling the `_with_override` seam directly with an explicit
    /// value proves the same thing without touching the environment at
    /// all, so there is nothing left to race.
    #[test]
    fn data_home_override_scopes_both_local_and_cache_dirs_under_one_root() {
        let root = PathBuf::from("/scratch/arama-data-home-scoping");

        let local = local_dir_with_override(Some(&root)).unwrap();
        let cache = cache_dir_with_override(Some(&root)).unwrap();

        assert_eq!(local, root.join(LOCAL_DIR));
        assert_eq!(cache, root.join(CACHE_DIR));
        assert_eq!(local.parent(), cache.parent());
    }
}
