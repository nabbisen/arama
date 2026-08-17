use std::{
    env,
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

fn data_home_override() -> Option<PathBuf> {
    env::var_os(DATA_HOME_ENV_VAR).map(PathBuf::from)
}

fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("", "", "arama").ok_or_else(|| {
        Error::other("could not resolve a platform data directory (no home directory found)")
    })
}

/// Where CLIP/wav2vec2 models and the legacy ffmpeg `bin/` live.
///
/// Platform data-local directory (RFC 041): `%LOCALAPPDATA%\arama\data` on
/// Windows, `~/Library/Application Support/arama` on macOS, `$XDG_DATA_HOME`
/// (or `~/.local/share`)`/arama` on Linux — never the executable's own
/// directory, which is unwritable once packaged, and never the roaming
/// profile, which large model downloads should not synchronise into.
pub fn local_dir() -> Result<PathBuf> {
    if let Some(root) = data_home_override() {
        return Ok(root.join(LOCAL_DIR));
    }
    Ok(project_dirs()?.data_local_dir().to_path_buf())
}

pub fn local_bin_dir() -> Result<PathBuf> {
    Ok(local_dir()?.join(BIN_DIR))
}

/// Where the thumbnail/embedding cache lives. Platform cache directory
/// (RFC 041): `%LOCALAPPDATA%\arama\cache` on Windows, `~/Library/Caches/arama`
/// on macOS, `$XDG_CACHE_HOME` (or `~/.cache`)`/arama` on Linux.
pub fn cache_dir() -> Result<PathBuf> {
    if let Some(root) = data_home_override() {
        return Ok(root.join(CACHE_DIR));
    }
    Ok(project_dirs()?.cache_dir().to_path_buf())
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

    /// The override must affect both locations identically, or a scratch
    /// profile using it would see models and cache split across two
    /// different roots - the exact "both locations populated" trap RFC 041
    /// exists to avoid, self-inflicted by the test isolation mechanism.
    #[test]
    fn data_home_override_scopes_both_local_and_cache_dirs_under_one_root() {
        // env::set_var/remove_var are process-global; serialise against any
        // other test touching this specific var by using a lock-free but
        // unique value per call and restoring afterward rather than
        // asserting on ambient state.
        let previous = env::var_os(DATA_HOME_ENV_VAR);
        let root = std::env::temp_dir().join(format!(
            "arama-dir-test-{}-{}",
            std::process::id(),
            "data-home-scoping"
        ));
        unsafe {
            env::set_var(DATA_HOME_ENV_VAR, &root);
        }

        let local = local_dir().unwrap();
        let cache = cache_dir().unwrap();

        unsafe {
            match &previous {
                Some(value) => env::set_var(DATA_HOME_ENV_VAR, value),
                None => env::remove_var(DATA_HOME_ENV_VAR),
            }
        }

        assert_eq!(local, root.join(LOCAL_DIR));
        assert_eq!(cache, root.join(CACHE_DIR));
        assert_eq!(local.parent(), cache.parent());
    }
}
