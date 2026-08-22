//! RFC 041: resolves and, on first run, migrates arama's three data
//! locations (settings, models, cache) to platform-correct homes, off the
//! executable's own directory (unwritable once packaged) and off the
//! current working directory (silently different settings depending on
//! how arama was launched).
//!
//! Two failure classes, deliberately different severities per RFC 017:
//!
//! - **Resolving or creating a location at all** ([`resolve_and_prepare_locations`])
//!   is a startup precondition. arama has nowhere to persist anything if
//!   this fails, so it is Fatal startup — the caller renders a blocking
//!   message instead of the normal shell.
//! - **Migrating existing data** into an already-creatable location
//!   ([`migrate_application_data`]) is Recoverable action: arama can still
//!   start at the new, possibly-empty location, so a failure here is a
//!   toast, not a refusal to run.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use app_json_settings::ConfigManager;
use arama_env::{
    DATA_HOME_ENV_VAR, Settings, cache_dir, legacy_cache_dir, legacy_local_dir, local_dir,
    validate_dir,
};
use arama_i18n::{t, t_with};

use super::StartupNotice;

pub(crate) struct ResolvedLocations {
    pub(crate) settings_manager: ConfigManager<Settings>,
    pub(crate) local_dir: PathBuf,
    pub(crate) cache_dir: PathBuf,
}

/// RFC 041 §7: "resolved locations are logged or otherwise discoverable, so
/// a failing run can be diagnosed without a debugger." The `NATIVE_SMOKE_`
/// prefix matches the convention already grepped for by
/// `.github/workflows/native-smoke.yaml`'s other steps.
pub(crate) fn describe_locations(locations: &ResolvedLocations) -> String {
    format!(
        "NATIVE_SMOKE_DATA_LOCATIONS_RESOLVED settings={} data={} cache={}",
        locations.settings_manager.folder_path().display(),
        locations.local_dir.display(),
        locations.cache_dir.display(),
    )
}

/// The unresolved counterpart: `resolve_and_prepare_locations` itself
/// already puts every reachable path into its error message (see below),
/// so this only needs to carry that message forward under the same
/// grep-able prefix.
pub(crate) fn describe_unresolved(message: &str) -> String {
    format!("NATIVE_SMOKE_DATA_LOCATIONS_UNRESOLVED: {message}")
}

/// Resolves and creates all three locations. Fatal (`Err`) if any cannot be
/// resolved or created at all — never silently falls back to "wherever the
/// filesystem happens to put it", which is the defect this RFC exists to
/// close.
pub(crate) fn resolve_and_prepare_locations() -> Result<ResolvedLocations, String> {
    let settings_manager = match std::env::var_os(DATA_HOME_ENV_VAR) {
        Some(root) => ConfigManager::new().with_root_dir(PathBuf::from(root)),
        None => ConfigManager::for_app("arama")
            .map_err(|err| format!("{}: {err}", t("startup.location_error.settings_resolve")))?,
    };
    validate_dir(settings_manager.folder_path()).map_err(|err| {
        format!(
            "{} ({}): {err}",
            t("startup.location_error.settings_create"),
            settings_manager.folder_path().display()
        )
    })?;

    let local_dir = local_dir()
        .map_err(|err| format!("{}: {err}", t("startup.location_error.data_resolve")))?;
    validate_dir(&local_dir).map_err(|err| {
        format!(
            "{} ({}): {err}",
            t("startup.location_error.data_create"),
            local_dir.display()
        )
    })?;

    let cache_dir = cache_dir()
        .map_err(|err| format!("{}: {err}", t("startup.location_error.cache_resolve")))?;
    validate_dir(&cache_dir).map_err(|err| {
        format!(
            "{} ({}): {err}",
            t("startup.location_error.cache_create"),
            cache_dir.display()
        )
    })?;

    Ok(ResolvedLocations {
        settings_manager,
        local_dir,
        cache_dir,
    })
}

/// First-run migration, in order of care: settings (smallest, most
/// precious), then models, then cache (both large; a move, not a
/// re-download or a re-index).
///
/// **Both-locations-populated is decided deliberately, not left to the
/// filesystem**: the new location wins and the old one is left untouched.
/// By the time this runs the new location has already been validated as
/// writable, so a populated new location means either a previous migration
/// already ran or the user has already been using the new layout —
/// overwriting it with older data from the broken location would be a
/// regression, not a migration.
pub(crate) fn migrate_application_data(locations: &ResolvedLocations) -> Vec<StartupNotice> {
    let mut notices = Vec::new();

    migrate_settings(&locations.settings_manager, &mut notices);
    migrate_directory(
        MigrationKind::Data,
        legacy_local_dir(),
        &locations.local_dir,
        &mut notices,
    );
    migrate_directory(
        MigrationKind::Cache,
        legacy_cache_dir(),
        &locations.cache_dir,
        &mut notices,
    );

    notices
}

/// Which of the two migrated directories `migrate_directory` is handling -
/// carries both the notice title and the translated noun used inside its
/// body (Task 034; previously a bare `&str`, translated via `capitalize`).
#[derive(Clone, Copy, Debug)]
enum MigrationKind {
    Data,
    Cache,
}

impl MigrationKind {
    fn title_key(self) -> &'static str {
        match self {
            MigrationKind::Data => "notice.data_migration_failed.title",
            MigrationKind::Cache => "notice.cache_migration_failed.title",
        }
    }

    fn noun(self) -> String {
        match self {
            MigrationKind::Data => t("notice.migration.kind_data"),
            MigrationKind::Cache => t("notice.migration.kind_cache"),
        }
    }
}

fn migrate_settings(new_manager: &ConfigManager<Settings>, notices: &mut Vec<StartupNotice>) {
    if new_manager.path().exists() {
        return; // new wins; old is left untouched
    }
    let old_manager = ConfigManager::<Settings>::new().at_current_dir();
    if !old_manager.path().exists() {
        return; // nothing to migrate
    }
    match old_manager.load() {
        Ok(settings) => {
            if let Err(err) = new_manager.save(&settings) {
                notices.push(StartupNotice::warning(
                    t("notice.settings_migration_failed.title"),
                    t_with(
                        "notice.settings_migration_failed.write_error.body",
                        &[
                            ("{path}", &new_manager.path().display().to_string()),
                            ("{err}", &err.to_string()),
                        ],
                    ),
                ));
            }
        }
        Err(err) => {
            notices.push(StartupNotice::warning(
                t("notice.settings_migration_failed.title"),
                t_with(
                    "notice.settings_migration_failed.read_error.body",
                    &[("{err}", &err.to_string())],
                ),
            ));
        }
    }
}

fn migrate_directory(
    kind: MigrationKind,
    legacy_dir: io::Result<PathBuf>,
    new_dir: &Path,
    notices: &mut Vec<StartupNotice>,
) {
    let legacy_dir = match legacy_dir {
        Ok(path) => path,
        Err(_) => return, // can't even resolve the exe path; nothing to migrate from
    };
    if !legacy_dir.exists() {
        return; // nothing to migrate
    }
    if directory_has_entries(new_dir) {
        return; // new wins; old is left untouched
    }
    if let Err(err) = move_or_copy_dir(&legacy_dir, new_dir) {
        let noun = kind.noun();
        notices.push(StartupNotice::warning(
            t(kind.title_key()),
            t_with(
                "notice.migration_failed.body",
                &[
                    ("{kind}", &noun),
                    ("{legacy}", &legacy_dir.display().to_string()),
                    ("{new}", &new_dir.display().to_string()),
                    ("{err}", &err.to_string()),
                ],
            ),
        ));
    }
}

fn directory_has_entries(path: &Path) -> bool {
    fs::read_dir(path).is_ok_and(|mut entries| entries.next().is_some())
}

/// Moves `from` to `to`, preferring an atomic rename (instant, and safe
/// even under concurrent access) and falling back to copy-then-delete only
/// when the two locations are on different filesystems, where a rename is
/// not possible at all.
///
/// The copy path is itself atomic from any observer's point of view: it
/// copies into a temporary sibling of `to` first and only renames that
/// sibling into `to`'s final name once the entire copy has succeeded, so a
/// migration interrupted partway never leaves a partially-populated
/// directory visible at `to` — `ModelContainer::ready_in` and similar
/// presence checks never observe a half-copied model.
fn move_or_copy_dir(from: &Path, to: &Path) -> io::Result<()> {
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent)?;
    }

    if fs::rename(from, to).is_ok() {
        return Ok(());
    }

    let tmp = tmp_sibling(to);
    let _ = fs::remove_dir_all(&tmp); // clear any stale attempt before retrying
    let copy_result = copy_dir_recursive(from, &tmp);
    if copy_result.is_err() {
        let _ = fs::remove_dir_all(&tmp);
        return copy_result;
    }
    fs::rename(&tmp, to)?;
    fs::remove_dir_all(from)?;
    Ok(())
}

fn tmp_sibling(to: &Path) -> PathBuf {
    let file_name = to
        .file_name()
        .map(|name| {
            let mut name = name.to_os_string();
            name.push(".arama-migrating-tmp");
            name
        })
        .unwrap_or_else(|| ".arama-migrating-tmp".into());
    to.with_file_name(file_name)
}

fn copy_dir_recursive(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest = to.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dest)?;
        } else if file_type.is_symlink() {
            // Neither models nor the cache are expected to contain
            // symlinks; skip rather than risk following one outside the
            // source tree.
            continue;
        } else {
            fs::copy(entry.path(), &dest)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describe_locations_carries_the_grep_marker_and_all_three_paths() {
        let root = std::env::temp_dir().join(format!(
            "arama-describe-locations-test-{}",
            std::process::id()
        ));
        let locations = ResolvedLocations {
            settings_manager: ConfigManager::new().with_root_dir(root.join("settings")),
            local_dir: root.join("data"),
            cache_dir: root.join("cache"),
        };

        let described = describe_locations(&locations);

        assert!(described.starts_with("NATIVE_SMOKE_DATA_LOCATIONS_RESOLVED "));
        assert!(described.contains(&root.join("settings").display().to_string()));
        assert!(described.contains(&root.join("data").display().to_string()));
        assert!(described.contains(&root.join("cache").display().to_string()));
    }

    #[test]
    fn describe_unresolved_carries_the_grep_marker_and_the_message() {
        let described = describe_unresolved("could not resolve the data location: no home dir");

        assert!(described.starts_with("NATIVE_SMOKE_DATA_LOCATIONS_UNRESOLVED: "));
        assert!(described.contains("could not resolve the data location"));
    }

    #[test]
    fn move_or_copy_dir_moves_a_real_directory_tree() {
        let root =
            std::env::temp_dir().join(format!("arama-migrate-test-{}-move", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let from = root.join("from");
        let to = root.join("to");
        fs::create_dir_all(from.join("nested")).unwrap();
        fs::write(from.join("a.txt"), b"a").unwrap();
        fs::write(from.join("nested/b.txt"), b"b").unwrap();

        move_or_copy_dir(&from, &to).unwrap();

        assert!(!from.exists());
        assert_eq!(fs::read_to_string(to.join("a.txt")).unwrap(), "a");
        assert_eq!(fs::read_to_string(to.join("nested/b.txt")).unwrap(), "b");
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn migrate_directory_does_nothing_when_new_location_already_has_entries() {
        let root = std::env::temp_dir().join(format!(
            "arama-migrate-test-{}-both-populated",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let legacy = root.join("legacy");
        let new = root.join("new");
        fs::create_dir_all(&legacy).unwrap();
        fs::write(legacy.join("old.txt"), b"old").unwrap();
        fs::create_dir_all(&new).unwrap();
        fs::write(new.join("current.txt"), b"current").unwrap();

        let mut notices = Vec::new();
        migrate_directory(MigrationKind::Data, Ok(legacy.clone()), &new, &mut notices);

        // New wins: untouched, and the legacy directory is left alone too.
        assert!(new.join("current.txt").exists());
        assert!(!new.join("old.txt").exists());
        assert!(legacy.join("old.txt").exists());
        assert!(notices.is_empty());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn migrate_directory_moves_when_new_location_is_empty() {
        let root = std::env::temp_dir().join(format!(
            "arama-migrate-test-{}-empty-new",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let legacy = root.join("legacy");
        let new = root.join("new");
        fs::create_dir_all(&legacy).unwrap();
        fs::write(legacy.join("model.bin"), b"weights").unwrap();
        fs::create_dir_all(&new).unwrap(); // exists, but empty

        let mut notices = Vec::new();
        migrate_directory(MigrationKind::Data, Ok(legacy.clone()), &new, &mut notices);

        assert!(new.join("model.bin").exists());
        assert!(!legacy.exists());
        assert!(notices.is_empty());
        fs::remove_dir_all(&root).unwrap();
    }

    /// Forces `move_or_copy_dir` to fail (the new location's parent is a
    /// *file*, so nothing can be created under it) and checks the
    /// resulting notice, for both `MigrationKind`s: no leftover
    /// `{placeholder}` token, and both paths actually present in the
    /// body, not just that it compiles.
    ///
    /// English-only, deliberately: `app`'s test binary runs 60+ tests in
    /// parallel, most of which assert exact English text without
    /// expecting `arama_i18n`'s global locale to move under them. An
    /// earlier version of this test looped over both locales; the same
    /// change elsewhere in this crate (`core.rs`, `update/cache.rs`) was
    /// found to race those tests when the full workspace suite ran
    /// repeatedly, so this one was written English-only from the start.
    /// `notice.migration_failed.body` (the key this reaches through
    /// `migrate_directory`) is verified in both locales in
    /// `arama-i18n`'s own, much smaller test binary instead
    /// (`crates/i18n/src/lib.rs`'s `task_034_*` tests).
    #[test]
    fn migrate_directory_failure_notice_reads_correctly_in_english() {
        for kind in [MigrationKind::Data, MigrationKind::Cache] {
            let root = std::env::temp_dir().join(format!(
                "arama-migrate-test-{}-failure-{kind:?}",
                std::process::id(),
            ));
            let _ = fs::remove_dir_all(&root);
            let legacy = root.join("legacy");
            fs::create_dir_all(&legacy).unwrap();
            fs::write(legacy.join("a.txt"), b"a").unwrap();

            let blocker = root.join("blocker");
            fs::write(&blocker, b"not a directory").unwrap();
            let new = blocker.join("new");

            let mut notices = Vec::new();
            migrate_directory(kind, Ok(legacy.clone()), &new, &mut notices);

            assert_eq!(
                notices.len(),
                1,
                "{kind:?}: expected exactly one failure notice"
            );
            let notice = &notices[0];
            assert!(!notice.title.is_empty(), "{kind:?}");
            assert!(
                !notice.body.contains('{'),
                "{kind:?}: leftover placeholder in {}",
                notice.body
            );
            assert!(
                notice.body.contains(&legacy.display().to_string()),
                "{kind:?}: legacy path missing from {}",
                notice.body
            );
            assert!(
                notice.body.contains(&new.display().to_string()),
                "{kind:?}: new path missing from {}",
                notice.body
            );

            fs::remove_dir_all(&root).unwrap();
        }
    }
}
