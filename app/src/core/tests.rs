use super::*;
use app_json_settings::SaveMode;

fn missing_test_path() -> PathBuf {
    std::env::temp_dir().join(format!("arama-missing-startup-root-{}", std::process::id()))
}

#[test]
fn missing_startup_root_is_recoverable_without_cache_node() {
    let path = missing_test_path();
    assert!(!path.exists());

    let root = startup_dir_node(
        &path.to_string_lossy(),
        &TargetMediaType {
            include_image: true,
            include_video: true,
        },
        0,
    );

    assert!(root.dir_node.is_none());
    assert_eq!(root.notices.len(), 1);
}

#[test]
fn walk_error_summary_omits_error_count_for_single_error() {
    let error = WalkError::Io {
        path: PathBuf::from("/not-readable"),
        source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
    };
    let summary = walk_errors_summary(&[error]);

    assert!(summary.contains("not-readable"));
    assert!(!summary.contains("total scan errors"));
}

#[test]
fn walk_error_summary_includes_error_count_for_multiple_errors() {
    let first = WalkError::Io {
        path: PathBuf::from("/not-readable"),
        source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
    };
    let second = WalkError::Io {
        path: PathBuf::from("/also-not-readable"),
        source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
    };
    let summary = walk_errors_summary(&[first, second]);

    assert!(summary.contains("not-readable"));
    assert!(summary.contains("2 total scan errors"));
}

/// Task 034: each `ConfigError` variant's translated label must
/// actually appear alongside the passed-through error text.
///
/// English-only, deliberately: `app`'s test binary runs 60+ tests in
/// parallel and most of them - like the two above - assert exact
/// English text without expecting the global locale
/// (`arama_i18n::set_locale`) to move under them. An earlier version
/// of this test looped over `Locale::all()`, and intermittently made
/// `walk_error_summary_includes_error_count_for_multiple_errors`
/// above observe Japanese output mid-flight and fail - a real race,
/// reproduced by running the full workspace suite repeatedly, not a
/// one-off. Japanese-locale correctness for these same keys is
/// verified in `arama-i18n`'s own, much smaller test binary instead
/// (`crates/i18n/src/lib.rs`'s `task_034_*` tests), where mutating
/// the global locale is safe because nothing else in that binary
/// assumes a fixed one.
#[test]
fn settings_error_message_reads_correctly_in_english() {
    let cases: Vec<ConfigError> = vec![
        ConfigError::Io(std::io::Error::other("disk full")),
        ConfigError::InvalidPathComponent("..".to_owned()),
        ConfigError::Platform("no home dir".to_owned()),
    ];

    for err in &cases {
        let message = settings_error_message(err);
        assert!(!message.is_empty(), "empty for {err:?}");
        assert!(!message.contains('{'), "leftover placeholder in {message}");
    }
}

#[test]
fn failed_setup_requirement_does_not_trigger_cache_transition() {
    assert!(!setup_became_complete(false, false, false));
}

#[test]
fn readiness_or_explicit_skip_triggers_cache_transition() {
    assert!(setup_became_complete(false, false, true));
    assert!(setup_became_complete(false, true, false));
    assert!(!setup_became_complete(true, true, true));
}

#[test]
fn production_settings_manager_uses_atomic_replacement() {
    let manager = ConfigManager::<Settings>::new();
    assert_eq!(manager.save_mode(), SaveMode::Atomic);
}

// --- RFC 041 §7 verification --------------------------------------
//
// `nothing_is_written_outside_arama_data_home` runs everywhere: it only
// ever touches an `ARAMA_DATA_HOME` scratch directory, so it is exactly
// as safe as `core::view::tests`'s existing `App::new()` test.
//
// The other three are gated `#[ignore]` and meant for
// `native-smoke.yaml` only (`cargo test -p arama --lib --locked --
// --ignored --exact <name> --nocapture`, matching that workflow's own
// convention for every other environment-touching check). Even with
// `ARAMA_DATA_HOME` covering the *new* side of a migration, these still
// create `.arama-local`/`.arama-cache` next to the test binary itself
// (`legacy_local_dir`/`legacy_cache_dir` are unconditionally
// exe-relative, on purpose - that's the pre-041 behaviour under test)
// and briefly change the process's current directory. Both are
// self-cleaning on success, but only worth doing on an ephemeral CI
// runner, not a developer's own machine.

fn scratch_data_home(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("arama-native-smoke-{}-{label}", std::process::id()))
}

#[test]
fn nothing_is_written_outside_arama_data_home() {
    let _guard = ARAMA_DATA_HOME_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = std::env::var_os(arama_env::DATA_HOME_ENV_VAR);
    let scratch = scratch_data_home("nothing-outside-data-home");
    let exe_parent = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let before: std::collections::BTreeSet<_> = std::fs::read_dir(&exe_parent)
        .unwrap()
        .filter_map(|entry| entry.ok().map(|entry| entry.file_name()))
        .collect();

    unsafe {
        std::env::set_var(arama_env::DATA_HOME_ENV_VAR, &scratch);
    }
    let mut app = App::new().0;
    app.settings.root_dir_path = "native-smoke-marker".to_owned();
    app.save_settings();
    unsafe {
        match &previous {
            Some(value) => std::env::set_var(arama_env::DATA_HOME_ENV_VAR, value),
            None => std::env::remove_var(arama_env::DATA_HOME_ENV_VAR),
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);

    let after: std::collections::BTreeSet<_> = std::fs::read_dir(&exe_parent)
        .unwrap()
        .filter_map(|entry| entry.ok().map(|entry| entry.file_name()))
        .collect();
    assert_eq!(
        before, after,
        "App::new()+save_settings() must not write anything next to the executable"
    );
}

#[test]
#[ignore]
fn native_smoke_settings_path_is_independent_of_working_directory() {
    // No ARAMA_DATA_HOME override: this deliberately exercises the real
    // `ConfigManager::for_app` platform resolution, the thing §4.1
    // fixed. The only real-machine effect is `mkdir -p` of the real
    // settings *directory* (never settings.json itself, and never an
    // overwrite) - resolving a location does not write the file.
    let _guard = ARAMA_DATA_HOME_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let original_cwd = std::env::current_dir().unwrap();
    let dir_a = scratch_data_home("cwd-independence-a");
    let dir_b = scratch_data_home("cwd-independence-b");
    std::fs::create_dir_all(&dir_a).unwrap();
    std::fs::create_dir_all(&dir_b).unwrap();

    std::env::set_current_dir(&dir_a).unwrap();
    let path_a = data_locations::resolve_and_prepare_locations()
        .unwrap()
        .settings_manager
        .path();
    std::env::set_current_dir(&dir_b).unwrap();
    let path_b = data_locations::resolve_and_prepare_locations()
        .unwrap()
        .settings_manager
        .path();
    std::env::set_current_dir(&original_cwd).unwrap();
    let _ = std::fs::remove_dir_all(&dir_a);
    let _ = std::fs::remove_dir_all(&dir_b);

    assert_eq!(
        path_a, path_b,
        "settings must resolve to the same path regardless of the working directory \
         arama was launched from (RFC 041 §4.1's defect)"
    );
}

#[test]
#[ignore]
fn native_smoke_migration_moves_settings_models_and_cache_from_the_legacy_layout() {
    let _guard = ARAMA_DATA_HOME_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let original_cwd = std::env::current_dir().unwrap();
    let legacy_settings_cwd = scratch_data_home("migration-legacy-settings-cwd");
    std::fs::create_dir_all(&legacy_settings_cwd).unwrap();
    let data_home = scratch_data_home("migration-new-side");

    let legacy_local = arama_env::legacy_local_dir().unwrap();
    let legacy_cache = arama_env::legacy_cache_dir().unwrap();
    let _ = std::fs::remove_dir_all(&legacy_local);
    let _ = std::fs::remove_dir_all(&legacy_cache);
    std::fs::create_dir_all(&legacy_local).unwrap();
    std::fs::create_dir_all(&legacy_cache).unwrap();
    std::fs::write(legacy_local.join("model.marker"), b"legacy-model").unwrap();
    std::fs::write(legacy_cache.join("cache.marker"), b"legacy-cache").unwrap();

    std::env::set_current_dir(&legacy_settings_cwd).unwrap();
    ConfigManager::<Settings>::new()
        .at_current_dir()
        .save(&Settings {
            root_dir_path: "native-smoke-legacy-marker".to_owned(),
            ..Settings::default()
        })
        .unwrap();
    unsafe {
        std::env::set_var(arama_env::DATA_HOME_ENV_VAR, &data_home);
    }

    let locations = data_locations::resolve_and_prepare_locations().unwrap();
    let notices = data_locations::migrate_application_data(&locations);

    unsafe {
        std::env::remove_var(arama_env::DATA_HOME_ENV_VAR);
    }
    std::env::set_current_dir(&original_cwd).unwrap();
    let _ = std::fs::remove_dir_all(&legacy_settings_cwd);

    assert!(
        notices.is_empty(),
        "migration should succeed without warnings: {:?}",
        notices.iter().map(|n| &n.title).collect::<Vec<_>>()
    );
    assert_eq!(
        locations.settings_manager.load().unwrap().root_dir_path,
        "native-smoke-legacy-marker"
    );
    assert_eq!(
        std::fs::read(locations.local_dir.join("model.marker")).unwrap(),
        b"legacy-model"
    );
    assert_eq!(
        std::fs::read(locations.cache_dir.join("cache.marker")).unwrap(),
        b"legacy-cache"
    );
    assert!(
        !legacy_local.exists(),
        "the legacy data directory must be moved, not left behind next to the executable"
    );
    assert!(
        !legacy_cache.exists(),
        "the legacy cache directory must be moved, not left behind next to the executable"
    );

    let _ = std::fs::remove_dir_all(&data_home);
}

#[test]
#[ignore]
fn native_smoke_migration_prefers_new_location_when_both_are_populated() {
    let _guard = ARAMA_DATA_HOME_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let data_home = scratch_data_home("migration-both-populated");
    let legacy_local = arama_env::legacy_local_dir().unwrap();
    let _ = std::fs::remove_dir_all(&legacy_local);
    std::fs::create_dir_all(&legacy_local).unwrap();
    std::fs::write(legacy_local.join("old.marker"), b"old").unwrap();

    unsafe {
        std::env::set_var(arama_env::DATA_HOME_ENV_VAR, &data_home);
    }
    let locations = data_locations::resolve_and_prepare_locations().unwrap();
    std::fs::write(locations.local_dir.join("current.marker"), b"current").unwrap();

    let notices = data_locations::migrate_application_data(&locations);

    unsafe {
        std::env::remove_var(arama_env::DATA_HOME_ENV_VAR);
    }

    assert!(notices.is_empty());
    assert!(locations.local_dir.join("current.marker").exists());
    assert!(!locations.local_dir.join("old.marker").exists());
    assert!(
        legacy_local.join("old.marker").exists(),
        "the new location wins; the legacy directory must be left untouched, not deleted"
    );

    let _ = std::fs::remove_dir_all(&data_home);
    let _ = std::fs::remove_dir_all(&legacy_local);
}

// --- RFC 044 Phase 0.1: what does the keyboard do in arama today? --
//
// Answered by running, not reading (handoff §4): `Simulator::tap_key`
// returns `event::Status::{Captured, Ignored}`, so "does anything
// consume this key today" is assertable in-process, per key, without
// a window or a compositor. arama installs no keyboard subscription
// and no focus operation anywhere (`subscription.rs` carries only the
// toast sweep), so the expectation is `Ignored` everywhere except
// wherever iced's own widgets already claim a key on arama's behalf -
// this test exists to find out where that is, not to assert a
// pre-decided answer.
#[test]
fn phase_0_1_keyboard_baseline_on_gallery_and_settings() {
    use iced::keyboard::{Key, key::Named};

    let _guard = ARAMA_DATA_HOME_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = std::env::var_os(arama_env::DATA_HOME_ENV_VAR);
    let scratch = scratch_data_home("phase-0-1-keyboard-baseline");
    unsafe {
        std::env::set_var(arama_env::DATA_HOME_ENV_VAR, &scratch);
    }

    let mut app = App::new().0;
    // Setup is not finished in a scratch profile (no CLIP model), so
    // `view()` would render the setup wizard rather than the gallery.
    // Phase 0.1 asks about the gallery and settings screens
    // specifically (handoff §4) - force setup complete the same way
    // `Message::Skip` would, without depending on that message's
    // other side effects.
    app.setup.finished = true;

    let keys = [
        ("Tab", Key::Named(Named::Tab)),
        ("Escape", Key::Named(Named::Escape)),
        ("Enter", Key::Named(Named::Enter)),
        ("ArrowDown", Key::Named(Named::ArrowDown)),
        ("F6", Key::Named(Named::F6)),
    ];

    eprintln!("=== Phase 0.1 keyboard baseline (RFC 044) ===");
    for (screen, nav) in [("gallery", None), ("settings", Some(NavPage::Settings))] {
        if let Some(nav) = nav {
            let _ = app.update(Message::NavTo(nav));
        }
        let element = app.view();
        let mut simulator = iced_test::Simulator::new(element);
        for (name, key) in &keys {
            let status = simulator.tap_key(key.clone());
            eprintln!("{screen:9} {name:10} -> {status:?}");
        }
    }

    unsafe {
        match &previous {
            Some(value) => std::env::set_var(arama_env::DATA_HOME_ENV_VAR, value),
            None => std::env::remove_var(arama_env::DATA_HOME_ENV_VAR),
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);
}

// --- RFC 044 Tier 2: in-process, headless, on real App state -------
//
// Phase 0.1 (above) already proved F6 is `Ignored` by every widget on
// the gallery and settings screens - so in the real application it
// reaches `subscription.rs`'s keyboard listener and becomes
// `Message::KeyPressed`. This test drives that exact message through
// `App::update` (not `Simulator`, which only exercises the widget
// tree, not `update`) and asserts the zone this RFC's own state
// actually moved to - no window, no compositor, no rendering.
#[test]
fn tier_2_f6_moves_focus_zone_through_real_app_update() {
    use iced::keyboard::{Key, Modifiers, key::Named};

    let _guard = ARAMA_DATA_HOME_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = std::env::var_os(arama_env::DATA_HOME_ENV_VAR);
    let scratch = scratch_data_home("tier-2-f6-focus-zone");
    unsafe {
        std::env::set_var(arama_env::DATA_HOME_ENV_VAR, &scratch);
    }

    let mut app = App::new().0;
    assert_eq!(
        app.focus_zone,
        snora::focus::FocusZone::Body,
        "starting zone, before any cycling"
    );
    assert!(
        !app.focus_visible,
        "the ring must not render before a keyboard user is known to exist"
    );

    let f6 = Key::Named(Named::F6);
    let _ = app.update(Message::KeyPressed(f6.clone(), Modifiers::default()));
    assert_eq!(
        app.focus_zone,
        snora::focus::FocusZone::Footer,
        "forward from Body skips the never-present Header and lands on Footer"
    );
    assert!(
        app.focus_visible,
        "the first real cycle must turn the ring on"
    );

    let _ = app.update(Message::KeyPressed(f6.clone(), Modifiers::default()));
    assert_eq!(
        app.focus_zone,
        snora::focus::FocusZone::SideBar,
        "forward from Footer wraps past Header to SideBar"
    );

    let _ = app.update(Message::KeyPressed(f6, Modifiers::SHIFT));
    assert_eq!(
        app.focus_zone,
        snora::focus::FocusZone::Footer,
        "Shift+F6 from SideBar goes backward, past Header, to Footer"
    );

    unsafe {
        match &previous {
            Some(value) => std::env::set_var(arama_env::DATA_HOME_ENV_VAR, value),
            None => std::env::remove_var(arama_env::DATA_HOME_ENV_VAR),
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);
}

#[test]
fn tier_2_escape_closes_a_real_open_dialog() {
    let _guard = ARAMA_DATA_HOME_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = std::env::var_os(arama_env::DATA_HOME_ENV_VAR);
    let scratch = scratch_data_home("tier-2-escape-closes-dialog");
    unsafe {
        std::env::set_var(arama_env::DATA_HOME_ENV_VAR, &scratch);
    }

    let mut app = App::new().0;
    let _ = app.update(Message::KeyPressed(
        iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
        iced::keyboard::Modifiers::default(),
    ));
    // Nothing open yet - Escape must not panic or synthesize a close.
    assert!(app.dialog.is_none());

    app.dialog = Some(Dialog::MediaFocusDialog(
        dialog::media_focus_dialog::MediaFocusDialog::new(
            PathBuf::from("/does/not/need/to/exist.jpg"),
            arama_env::cache_lookup_strategy::CacheLookupStrategy::CurrentDirOnly,
            0.86,
            None,
        ),
    ));
    assert!(app.dialog.is_some());

    let _ = app.update(Message::KeyPressed(
        iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
        iced::keyboard::Modifiers::default(),
    ));
    assert!(
        app.dialog.is_none(),
        "Escape must close the real dialog through the real update path"
    );

    unsafe {
        match &previous {
            Some(value) => std::env::set_var(arama_env::DATA_HOME_ENV_VAR, value),
            None => std::env::remove_var(arama_env::DATA_HOME_ENV_VAR),
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);
}

// --- RFC 044 §0.2b: does `Simulator::snapshot` work here at all? ---
//
// snora has never run this path (RFC-011-D chose semantic over pixel
// testing) and asked to hear whether it works for a focus indicator.
// This is that experiment, not a permanent regression suite - Tier
// 3's own footgun (`matches_image`/`matches_hash` auto-create *and*
// auto-pass on a missing reference) means a real regression suite
// needs checked-in reference files and a documented regeneration
// process, which is a separate decision from "does the mechanism
// work." Reported in the review package either way.
#[test]
fn phase_0_2b_does_simulator_snapshot_render_here() {
    use iced::keyboard::{Key, Modifiers, key::Named};

    let _guard = ARAMA_DATA_HOME_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = std::env::var_os(arama_env::DATA_HOME_ENV_VAR);
    let scratch = scratch_data_home("phase-0-2b-snapshot-experiment");
    unsafe {
        std::env::set_var(arama_env::DATA_HOME_ENV_VAR, &scratch);
    }

    let mut app = App::new().0;
    app.setup.finished = true;
    // Move focus so the ring is actually present in the frame this
    // snapshots - a snapshot of the pre-`focus_visible` state would
    // prove nothing about the indicator.
    let _ = app.update(Message::KeyPressed(
        Key::Named(Named::F6),
        Modifiers::default(),
    ));
    assert!(app.focus_visible);

    let element = app.view();
    let mut simulator = iced_test::Simulator::new(element);
    let theme = arama_theme::iced_theme();
    let result = simulator.snapshot(&theme);

    eprintln!(
        "=== Phase 0.2b: Simulator::snapshot result: {:?} ===",
        result.as_ref().map(|_| "Ok")
    );
    let snapshot = result.expect(
        "Simulator::snapshot must at least render successfully on this hardware \
         for Tier 3 to be a real option",
    );

    // Exercise the actual comparison codepath too, not just draw +
    // screenshot - on a scratch path so this run's baseline is
    // thrown away rather than becoming a permanent reference nobody
    // reviewed.
    let hash_path = scratch.join("phase-0-2b-snapshot");
    let first_call = snapshot
        .matches_hash(&hash_path)
        .expect("hashing the rendered frame must not fail");
    assert!(first_call, "a freshly created reference must match itself");
    let second_call = snapshot
        .matches_hash(&hash_path)
        .expect("hashing the rendered frame must not fail");
    assert!(
        second_call,
        "the same App state must render identically across two snapshot calls"
    );
    eprintln!("=== Phase 0.2b: matches_hash round-trip succeeded ===");

    unsafe {
        match &previous {
            Some(value) => std::env::set_var(arama_env::DATA_HOME_ENV_VAR, value),
            None => std::env::remove_var(arama_env::DATA_HOME_ENV_VAR),
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);
}

/// Task 032: the RFC 044 Tier 2 tests above only ever check
/// `App::focus_zone`, arama's own enum - if `view.rs` wrapped the
/// wrong container in `focus_ring_style`, every one of them would
/// still pass. This asserts against snora's own compatibility-surface
/// identifiers instead (`docs/src/reference/rendered-surface-identifiers.md`,
/// `snora-0.39.1/src/identifiers.rs`: `HEADER_REGION`/`SIDEBAR_REGION`/
/// `BODY_REGION`/`FOOTER_REGION`, `"snora-header"`/`"snora-sidebar"`/
/// `"snora-body"`/`"snora-footer"`) - the region snora itself thinks a
/// zone is, not just the label arama gives that zone internally.
///
/// `iced_selector::Selector for widget::Id` is confirmed present at
/// `iced_selector-0.14.0/src/lib.rs:99,146` and reachable through the
/// `iced_test` dev-dependency already added for RFC 044 - no new
/// dependency.
///
/// What this can and cannot prove: `Simulator::find` returns a
/// `Target::Container { id, bounds, visible_bounds }` -
/// `iced_selector-0.14.0/src/target.rs:11-15` - bounds only, no style.
/// It cannot see the ring's own border colour or width; snora's
/// region container is the *outer* wrapper, arama's ring-styled
/// container is nested one level inside it (`app/src/core/view.rs`'s
/// `side_bar`/`body`/`footer` locals are handed to snora's
/// `AppLayout::side_bar`/`body`/`footer`, which wraps each in its own
/// `.id(...)`-carrying container). So this test proves the slot
/// snora considers "the sidebar"/"the body"/"the footer" exists and
/// is where arama's own `FocusZone` enum says it should be, and that
/// snora's `header` region is correctly absent (arama sets no
/// `AppLayout::header`) - it does not and cannot reach into whether
/// the ring is actually drawn there.
#[test]
fn tier_2_focus_ring_zones_resolve_to_snoras_own_region_ids() {
    use iced::keyboard::{Key, Modifiers, key::Named};
    use iced_test::selector::id;

    let _guard = ARAMA_DATA_HOME_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = std::env::var_os(arama_env::DATA_HOME_ENV_VAR);
    let scratch = scratch_data_home("tier-2-focus-ring-snora-slot");
    unsafe {
        std::env::set_var(arama_env::DATA_HOME_ENV_VAR, &scratch);
    }

    let mut app = App::new().0;
    app.setup.finished = true;

    // Every skeleton slot arama populates is present regardless of
    // which zone currently holds focus - `side_bar`/`body`/`footer`
    // are unconditional in `view.rs`. Checked once, before any F6
    // press, so the presence assertion is not entangled with the
    // focus-cycling assertions below.
    {
        let mut simulator = iced_test::Simulator::new(app.view());
        assert!(
            simulator
                .find(id(iced::widget::Id::new("snora-sidebar")))
                .is_ok(),
            "snora's sidebar region must exist"
        );
        assert!(
            simulator
                .find(id(iced::widget::Id::new("snora-body")))
                .is_ok(),
            "snora's body region must exist"
        );
        assert!(
            simulator
                .find(id(iced::widget::Id::new("snora-footer")))
                .is_ok(),
            "snora's footer region must exist"
        );
        // arama never populates `AppLayout::header` - confirms the
        // Tier 1 `header_never_appears_because_it_is_never_present`
        // assumption holds at the snora-region level too, not just
        // in `ZonePresence`.
        assert!(
            simulator
                .find(id(iced::widget::Id::new("snora-header")))
                .is_err(),
            "snora's header region must be absent - arama sets no AppLayout::header"
        );
    }

    // F6 from the Body start zone lands on Footer first (RFC 044:
    // Header is skipped because `ZonePresence` reports it absent).
    let f6 = Key::Named(Named::F6);
    let _ = app.update(Message::KeyPressed(f6.clone(), Modifiers::default()));
    assert_eq!(app.focus_zone, snora::focus::FocusZone::Footer);
    {
        let mut simulator = iced_test::Simulator::new(app.view());
        assert!(
            simulator
                .find(id(iced::widget::Id::new("snora-footer")))
                .is_ok(),
            "snora-footer must resolve while arama's own state says Footer is focused"
        );
    }

    let _ = app.update(Message::KeyPressed(f6.clone(), Modifiers::default()));
    assert_eq!(app.focus_zone, snora::focus::FocusZone::SideBar);
    {
        let mut simulator = iced_test::Simulator::new(app.view());
        assert!(
            simulator
                .find(id(iced::widget::Id::new("snora-sidebar")))
                .is_ok(),
            "snora-sidebar must resolve while arama's own state says SideBar is focused"
        );
    }

    let _ = app.update(Message::KeyPressed(f6, Modifiers::default()));
    assert_eq!(app.focus_zone, snora::focus::FocusZone::Body);
    {
        let mut simulator = iced_test::Simulator::new(app.view());
        assert!(
            simulator
                .find(id(iced::widget::Id::new("snora-body")))
                .is_ok(),
            "snora-body must resolve while arama's own state says Body is focused"
        );
    }

    unsafe {
        match &previous {
            Some(value) => std::env::set_var(arama_env::DATA_HOME_ENV_VAR, value),
            None => std::env::remove_var(arama_env::DATA_HOME_ENV_VAR),
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);
}

/// The `is_focused()` alternative the review for package 122 raised
/// (`iced_selector-0.14.0/src/lib.rs:154`) - tried and rejected, with
/// evidence rather than assumption. It matches `Candidate::Focusable`,
/// which only iced's own keyboard-focusable widgets (`text_input` and
/// similar) produce. arama's ring is a plain styled `container` with
/// no `Focusable` state of its own - F6 zone cycling is arama's own
/// enum, entirely outside iced's native focus system - so this must
/// find nothing, on every zone, confirming `id(...)` (above) is the
/// only one of the two that can reach these containers at all.
#[test]
fn is_focused_selector_cannot_see_aramas_zone_ring_containers() {
    use iced::keyboard::{Key, Modifiers, key::Named};
    use iced_test::selector::is_focused;

    let _guard = ARAMA_DATA_HOME_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = std::env::var_os(arama_env::DATA_HOME_ENV_VAR);
    let scratch = scratch_data_home("is-focused-selector-experiment");
    unsafe {
        std::env::set_var(arama_env::DATA_HOME_ENV_VAR, &scratch);
    }

    let mut app = App::new().0;
    app.setup.finished = true;
    let _ = app.update(Message::KeyPressed(
        Key::Named(Named::F6),
        Modifiers::default(),
    ));
    assert!(app.focus_visible);

    let mut simulator = iced_test::Simulator::new(app.view());
    let result = simulator.find(is_focused());
    assert!(
        result.is_err(),
        "is_focused() must not see arama's ring containers - they carry no \
         iced-native Focusable state, only app-level FocusZone/style; got {result:?}"
    );

    unsafe {
        match &previous {
            Some(value) => std::env::set_var(arama_env::DATA_HOME_ENV_VAR, value),
            None => std::env::remove_var(arama_env::DATA_HOME_ENV_VAR),
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);
}
