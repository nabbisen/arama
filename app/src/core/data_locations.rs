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
mod tests;
