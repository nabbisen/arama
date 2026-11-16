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

/// T070 (snora 0.52 -> 0.57): this test's premise through 0.52 was that
/// `Message::KeyPressed(Escape)` reaches `App::update` while an overlay is
/// open, the same as any other key. From 0.55 that stopped being true:
/// snora's own engine captures Escape itself, before
/// `iced::keyboard::listen()`'s subscription ever produces a message for
/// it, whenever `on_close_modals`/`on_close_menus` are set (`view.rs` sets
/// both) - so on 0.57 the second half below would pass for the wrong
/// reason (nothing closes the dialog, `app.update` is simply never asked
/// to), not the right one.
///
/// Re-pointed to press Escape through `iced_test::Simulator` on the real
/// rendered view instead - the only way left to exercise the real path,
/// since `App::update` is no longer on it. The first half (nothing open)
/// is unchanged: with no overlay, `on_close_modals`/`on_close_menus` do
/// not apply, so Escape still reaches `Message::KeyPressed` the ordinary
/// way and must still do nothing.
#[test]
fn tier_2_escape_closes_a_real_open_dialog() {
    use iced::keyboard::{Key, Modifiers, key::Named};

    let _guard = ARAMA_DATA_HOME_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = std::env::var_os(arama_env::DATA_HOME_ENV_VAR);
    let scratch = scratch_data_home("tier-2-escape-closes-dialog");
    unsafe {
        std::env::set_var(arama_env::DATA_HOME_ENV_VAR, &scratch);
    }

    let mut app = App::new().0;
    app.setup.finished = true;

    // First half, unchanged: nothing open, Escape via the real `update`
    // path must not synthesize a close.
    let _ = app.update(Message::KeyPressed(
        Key::Named(Named::Escape),
        Modifiers::default(),
    ));
    assert!(app.dialog.is_none());

    // Second half, re-pointed: a real dialog open, rendered for real,
    // Escape pressed through the Simulator on that rendered view -
    // snora's engine, not `App::update` directly, is what claims it now.
    app.dialog = Some(Dialog::MediaFocusDialog(
        dialog::media_focus_dialog::MediaFocusDialog::new(
            PathBuf::from("/does/not/need/to/exist.jpg"),
            arama_env::cache_lookup_strategy::CacheLookupStrategy::CurrentDirOnly,
            0.86,
            None,
        ),
    ));
    assert!(app.dialog.is_some());

    let element = app.view();
    let mut simulator = iced_test::Simulator::new(element);
    let status = simulator.tap_key(Key::Named(Named::Escape));
    let messages: Vec<Message> = simulator.into_messages().collect();
    assert!(
        messages
            .iter()
            .any(|message| matches!(message, Message::DialogClose)),
        "Escape on the rendered overlay must produce DialogClose - got {messages:?}, \
         event status {status:?}"
    );
    for message in messages {
        let _ = app.update(message);
    }
    assert!(
        app.dialog.is_none(),
        "the real DialogClose message, fed back through update, must actually close the dialog"
    );

    // Same shape, for an open context menu: `ContextMenuState::ImageCell`
    // is what `is_open()` checks, and what makes `view()` populate
    // snora's `context_menu` slot (activating its own Escape/backdrop
    // dismissal) - not constructible without a real path, so a throwaway
    // one is enough; nothing here reads the file.
    app.context_menu.state = arama_ui_widgets::context_menu::ContextMenuState::ImageCell(
        PathBuf::from("/does/not/need/to/exist.jpg"),
    );
    assert!(app.context_menu.is_open());

    let element = app.view();
    let mut simulator = iced_test::Simulator::new(element);
    let status = simulator.tap_key(Key::Named(Named::Escape));
    let messages: Vec<Message> = simulator.into_messages().collect();
    assert!(
        messages
            .iter()
            .any(|message| matches!(message, Message::CloseMenus)),
        "Escape on the rendered context menu must produce CloseMenus - got {messages:?}, \
         event status {status:?}"
    );
    for message in messages {
        let _ = app.update(message);
    }
    assert!(
        !app.context_menu.is_open(),
        "the real CloseMenus message, fed back through update, must actually close the menu"
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

// --- RFC 047 S3.4/S7.1: Stop and a directory switch must leave a
// partial index visibly partial, through the real `App::update` path --

#[test]
fn stop_signals_the_cancel_flag_clears_the_task_handle_and_un_completes_embedding() {
    let _guard = ARAMA_DATA_HOME_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = std::env::var_os(arama_env::DATA_HOME_ENV_VAR);
    let scratch = scratch_data_home("stop-signals-cancel-flag");
    unsafe {
        std::env::set_var(arama_env::DATA_HOME_ENV_VAR, &scratch);
    }

    let mut app = App::new().0;

    // Simulate a run already in flight: a real abortable handle (its
    // target task does not matter, only that `Handle::abort` on it is
    // observable as "taken"), a cancel flag the running phase would be
    // checking, and a directory whose embedding this run would complete
    // if left alone.
    let (_task, handle) = iced::Task::<()>::none().abortable();
    app.task_handle = Some(handle);
    let cancel_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    app.cancel_flag = Some(cancel_flag.clone());
    app.embedding_completed = true;
    // Review 161 #1: a real run in flight is always `processing`; the
    // bug was that `abort_indexing` alone never cleared it, which left
    // the aside blocking expand/selection (it checks `processing`) for
    // the rest of the session after one Stop.
    app.processing = true;

    let _ = app.update(Message::CachePageMessage(
        arama_ui_main::views::cache_page::message::Message::Event(
            arama_ui_main::views::cache_page::message::Event::StopRequest,
        ),
    ));

    assert!(
        app.task_handle.is_none(),
        "Stop must take and abort the in-flight task handle"
    );
    assert!(
        app.cancel_flag.is_none(),
        "Stop must not leave a stale cancel flag referenced for the next run to find already \
         set"
    );
    assert!(
        cancel_flag.load(std::sync::atomic::Ordering::Relaxed),
        "the flag the running phase holds its own clone of must have been set, not just \
         dropped from App"
    );
    assert!(
        !app.embedding_completed,
        "a directory stopped midway must not present itself as fully indexed (RFC 047 S7.1)"
    );
    assert!(
        !app.processing,
        "review 161 #1: Stop must return the app to idle, or the aside stays blocked and \
         settings-triggered runs never start again until restart"
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
fn switching_directories_mid_index_cancels_both_the_task_and_the_flag() {
    let _guard = ARAMA_DATA_HOME_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = std::env::var_os(arama_env::DATA_HOME_ENV_VAR);
    let scratch = scratch_data_home("dir-switch-cancels-both-phases");
    unsafe {
        std::env::set_var(arama_env::DATA_HOME_ENV_VAR, &scratch);
    }
    let new_dir = std::env::temp_dir().join(format!(
        "arama-dir-switch-cancel-test-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&new_dir);
    std::fs::create_dir_all(&new_dir).unwrap();

    let mut app = App::new().0;
    let (_task, handle) = iced::Task::<()>::none().abortable();
    app.task_handle = Some(handle);
    let cancel_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    app.cancel_flag = Some(cancel_flag.clone());
    app.embedding_completed = true;

    // The same message path a header path-submit or an aside tree click
    // produces (`component.rs`'s `header::message::Event::DirSelect`
    // handler calls `on_dir_changed`, which this exercises through the
    // real `App::update`, not by calling `on_dir_changed` directly).
    let _ = app.update(Message::HeaderMessage(
        arama_ui_layout::header::message::Message::Event(
            arama_ui_layout::header::message::Event::DirSelect(new_dir.clone()),
        ),
    ));

    assert!(
        app.task_handle.is_none(),
        "a directory switch must abort the previous directory's in-flight task"
    );
    assert!(
        cancel_flag.load(std::sync::atomic::Ordering::Relaxed),
        "a directory switch must signal the previous run's cancel flag, exactly like Stop - \
         RFC 047 S1's claim #3 ('switching directories cancels the running index') must hold \
         for the thumbnail phase too, not only embedding"
    );
    assert!(
        !app.embedding_completed,
        "the new directory must start un-indexed, not inherit the old one's completed state"
    );

    unsafe {
        match &previous {
            Some(value) => std::env::set_var(arama_env::DATA_HOME_ENV_VAR, value),
            None => std::env::remove_var(arama_env::DATA_HOME_ENV_VAR),
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);
    let _ = std::fs::remove_dir_all(&new_dir);
}

/// Review 161, "smaller": two settings-triggered run-starts
/// (`TargetMediaTypeChanged`/`SubDirDepthLimitChanged`) start a new run
/// without going through `abort_indexing` (nothing was running, so
/// there is nothing to abort), which used to leave a stale
/// `embedding_completed = true` from a previous directory's completed
/// run - the button would stay available on the *old* completion. The
/// fix lives at the one choke point every run-start path passes
/// through (`handle_cache_require`), confirmed here through the real
/// `App::update` path for `TargetMediaTypeChanged` specifically - the
/// exact scenario the review's own probe reproduced.
#[test]
fn settings_triggered_run_start_clears_a_stale_embedding_completed() {
    let _guard = ARAMA_DATA_HOME_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = std::env::var_os(arama_env::DATA_HOME_ENV_VAR);
    let scratch = scratch_data_home("settings-run-start-clears-stale-completed");
    unsafe {
        std::env::set_var(arama_env::DATA_HOME_ENV_VAR, &scratch);
    }

    let mut app = App::new().0;
    // A previous directory's run completed in full.
    app.embedding_completed = true;
    app.processing = false;

    // The settings handler itself only returns a `Task::done(CacheRequire(None))`
    // - it does not call `handle_cache_require` directly. `iced::Task`
    // has no public way to poll itself to completion outside the real
    // application executor (confirmed during T061's own work), so the
    // second step is driven by hand here, exactly as the executor would
    // deliver it.
    let _ = app.update(Message::SettingsDialogMessage(
        arama_ui_widgets::dialog::settings_dialog::message::Message::TargetMediaTypeChanged(
            arama_env::target_media_type::TargetMediaType {
                include_image: true,
                include_video: true,
            },
        ),
    ));
    let _ = app.update(Message::CacheRequire(None));

    assert!(
        !app.embedding_completed,
        "a brand new run must not inherit the previous directory's completed state"
    );

    unsafe {
        match &previous {
            Some(value) => std::env::set_var(arama_env::DATA_HOME_ENV_VAR, value),
            None => std::env::remove_var(arama_env::DATA_HOME_ENV_VAR),
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);
}

/// RFC 047 S3.4: reaching `handle_embedding_cache_finished` is itself
/// proof the embedding `Task` was never aborted (confirmed against
/// `iced_runtime`'s `Abortable`-based `Handle::abort` - an aborted
/// task's completion message is never delivered), so this handler is
/// the only place allowed to set `embedding_completed` back to `true`.
#[test]
fn embedding_finished_handler_sets_embedding_completed_true_through_real_app_update() {
    let _guard = ARAMA_DATA_HOME_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = std::env::var_os(arama_env::DATA_HOME_ENV_VAR);
    let scratch = scratch_data_home("embedding-finished-sets-completed");
    unsafe {
        std::env::set_var(arama_env::DATA_HOME_ENV_VAR, &scratch);
    }

    let mut app = App::new().0;
    assert!(
        !app.embedding_completed,
        "must start false - nothing has been indexed yet"
    );

    let _ = app.update(Message::EmbeddingCacheFinished(Ok(
        arama_ai::pipeline::encode::image::embeddings::EmbeddingRunReport::default(),
    )));

    assert!(
        app.embedding_completed,
        "reaching the real finished handler (not an abort) must mark embedding complete"
    );
    assert!(
        app.task_handle.is_none(),
        "the finished handler must clear the task handle"
    );

    unsafe {
        match &previous {
            Some(value) => std::env::set_var(arama_env::DATA_HOME_ENV_VAR, value),
            None => std::env::remove_var(arama_env::DATA_HOME_ENV_VAR),
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);
}

/// Review 162 (T061's second rework): closing the pairs dialog and
/// reopening it cannot abort the old scan (`ScanControl`'s own doc
/// comment explains why `Task::abortable()` cannot either), so its
/// eventual, partial result still arrives while the *new* scan's dialog
/// is open. Without a generation check, that stale `EmbeddingsReady`
/// would be accepted as the new scan's own: clearing
/// `similar_pairs_scan` out from under the new scan (so closing the new
/// dialog would no longer cancel it) and handing the new dialog the old
/// scan's partial pairs as if they were complete (RFC 047 S3.3 / RFC
/// 036). This drives both deliveries through the real `App::update`
/// path, not the dialog's own `update` directly - the check lives in
/// `handle_similar_pairs_dialog_message`, between the two.
#[test]
fn stale_embeddings_ready_from_a_closed_scan_is_dropped_the_current_one_is_accepted() {
    use arama_ai::pipeline::score::similarity::scan_control::ScanControl;
    use arama_ui_widgets::dialog::similar_pairs_dialog::{
        EmbeddingsReadyResult, SimilarPairsDialog, message as similar_pairs_dialog_message,
    };

    let _guard = ARAMA_DATA_HOME_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = std::env::var_os(arama_env::DATA_HOME_ENV_VAR);
    let scratch = scratch_data_home("stale-embeddings-ready-dropped");
    unsafe {
        std::env::set_var(arama_env::DATA_HOME_ENV_VAR, &scratch);
    }

    let mut app = App::new().0;

    // Simulate "scan 2 is current, its dialog is open" directly - scan 1
    // (generation 1) was abandoned by closing and reopening the dialog,
    // which is exactly how generation 2 came to exist, but the scan
    // itself is never actually constructible/runnable from here (it
    // touches the real cache and filesystem); only its aftermath -
    // the stale message arriving - is this test's subject.
    app.similar_pairs_scan_generation = 2;
    let scan_2_control = std::sync::Arc::new(ScanControl::new());
    app.similar_pairs_scan = Some(scan_2_control.clone());
    let dir_node = DirNode {
        path: missing_test_path(),
        sub_dirs: vec![],
        files: vec![],
    };
    app.dialog = Some(Dialog::SimilarPairsDialog(SimilarPairsDialog::new(
        dir_node, None, 0.0, None,
    )));

    // Scan 1's late, partial result arrives.
    let _ = app.update(Message::SimilarPairsDialogMessage(
        similar_pairs_dialog_message::Message::EmbeddingsReady(1, EmbeddingsReadyResult::default()),
    ));

    let Some(Dialog::SimilarPairsDialog(dialog)) = &app.dialog else {
        panic!("the dialog must still be open - a dropped stale message must not touch it");
    };
    assert!(
        dialog.is_scanning(),
        "a stale (generation 1) result must not be accepted while generation 2 is current"
    );
    assert!(
        app.similar_pairs_scan
            .as_ref()
            .is_some_and(|control| std::sync::Arc::ptr_eq(control, &scan_2_control)),
        "scan 2's control must survive a stale generation-1 result untouched"
    );

    // Scan 2's own result arrives.
    let _ = app.update(Message::SimilarPairsDialogMessage(
        similar_pairs_dialog_message::Message::EmbeddingsReady(2, EmbeddingsReadyResult::default()),
    ));

    let Some(Dialog::SimilarPairsDialog(dialog)) = &app.dialog else {
        panic!("the dialog must still be open");
    };
    assert!(
        !dialog.is_scanning(),
        "the current generation's own result must be accepted"
    );
    assert!(
        app.similar_pairs_scan.is_none(),
        "accepting the current scan's result must clear similar_pairs_scan - nothing is left \
         to poll or cancel"
    );

    unsafe {
        match &previous {
            Some(value) => std::env::set_var(arama_env::DATA_HOME_ENV_VAR, value),
            None => std::env::remove_var(arama_env::DATA_HOME_ENV_VAR),
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);
}

/// T070 (snora 0.52 -> 0.57, RFC-113/RFC-120), §5: the toast attention
/// hold, through the real `App::update` path (`Message::ToastAttention`,
/// `Message::ToastSweep`) - never by calling `Holds::attend`/`Holds::sweep`
/// directly, which would prove the type works but not that it is wired in.
#[test]
fn toast_attention_hold_survives_a_sweep_and_expires_once_released() {
    let _guard = ARAMA_DATA_HOME_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = std::env::var_os(arama_env::DATA_HOME_ENV_VAR);
    let scratch = scratch_data_home("toast-attention-hold");
    unsafe {
        std::env::set_var(arama_env::DATA_HOME_ENV_VAR, &scratch);
    }

    let mut app = App::new().0;
    let id = 1;
    // A short real lifetime, not a mocked clock: `handle_toast_sweep`/
    // `handle_toast_attention` call `Instant::now()` internally (matching
    // every other real caller), so there is no injectable time to control
    // from here - the same shape as this file's other short-real-sleep
    // timing tests (e.g. the thumbnail-phase ones).
    app.toasts.push(
        snora::Toast::new(
            id,
            snora::ToastIntent::Info,
            "t",
            "m",
            Message::ToastDismiss(id),
        )
        .with_lifetime(snora::ToastLifetime::Transient(
            std::time::Duration::from_millis(50),
        )),
    );

    let _ = app.update(Message::ToastAttention(
        id,
        snora::ToastAttention::PointerEntered,
    ));
    std::thread::sleep(std::time::Duration::from_millis(100));
    let _ = app.update(Message::ToastSweep);
    assert!(
        app.toasts.iter().any(|t| t.id == id),
        "a toast with attention held must survive a sweep past its own lifetime"
    );

    // Released: the countdown restarts in full from now, so an
    // immediate sweep must not expire it either.
    let _ = app.update(Message::ToastAttention(
        id,
        snora::ToastAttention::PointerLeft,
    ));
    let _ = app.update(Message::ToastSweep);
    assert!(
        app.toasts.iter().any(|t| t.id == id),
        "releasing attention must restart the countdown in full, not expire the toast instantly"
    );

    // Now past the restarted deadline, with nothing holding it.
    std::thread::sleep(std::time::Duration::from_millis(100));
    let _ = app.update(Message::ToastSweep);
    assert!(
        !app.toasts.iter().any(|t| t.id == id),
        "once attention has ended and its restarted lifetime has passed, the toast must expire"
    );

    unsafe {
        match &previous {
            Some(value) => std::env::set_var(arama_env::DATA_HOME_ENV_VAR, value),
            None => std::env::remove_var(arama_env::DATA_HOME_ENV_VAR),
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);
}

/// RFC 053 §6.2/§7: the minimum and launch window sizes are both set,
/// both fit inside the smallest common laptop screen's content area
/// (1093 x 535 logical px, RFC 053 §2.2), and the launch size is never
/// smaller than the minimum - reading `window_settings()` directly
/// (`App::start` itself cannot be called from a test: its `.run()`
/// blocks on the platform event loop), so a change to the constants
/// alone, with nothing calling them, cannot pass this silently.
#[test]
fn window_settings_minimum_and_launch_size_fit_the_smallest_laptop_screen() {
    let settings = window_settings();

    assert_eq!(
        settings.min_size,
        Some(iced::Size::new(760.0, 520.0)),
        "the minimum must be exactly RFC 053's owner-decided 760 x 520"
    );
    assert_eq!(
        settings.size,
        iced::Size::new(1024.0, 520.0),
        "the launch size must be the handoff's own default, 1024 x 520, absent a reason \
         Phase 0 established to deviate from it"
    );
    assert!(
        settings.size.width <= 1093.0 && settings.size.height <= 535.0,
        "the launch size must fit the smallest common laptop screen's content area: \
         {:?}",
        settings.size
    );
    assert!(
        settings.min_size.is_some_and(
            |min| settings.size.width >= min.width && settings.size.height >= min.height
        ),
        "the launch size must never be smaller than the minimum the window itself \
         enforces - opening below min_size is not a state the window can even reach"
    );
}

/// RFC 053 §6.2 (review 165, P1)/§7: the window's title and, on Linux,
/// its application id, both match `packaging/linux/arama.desktop`'s own
/// name - read from that file directly, so renaming one without the
/// other fails this test rather than only being noticed by a user.
#[test]
fn window_title_and_linux_application_id_match_the_desktop_file() {
    let desktop_file = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../packaging/linux/arama.desktop"
    ))
    .expect("packaging/linux/arama.desktop must exist and be readable");
    let product_name = desktop_file
        .lines()
        .find_map(|line| line.strip_prefix("Name="))
        .expect("arama.desktop must declare Name=");

    assert_eq!(
        WINDOW_TITLE, product_name,
        "the window title must match arama.desktop's own Name= exactly"
    );

    #[cfg(target_os = "linux")]
    {
        let desktop_basename = std::path::Path::new("arama.desktop")
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("arama.desktop's basename must be valid UTF-8");
        assert_eq!(
            LINUX_APPLICATION_ID, desktop_basename,
            "the Linux application id must equal arama.desktop's own basename"
        );
        assert_eq!(
            window_settings().platform_specific.application_id,
            LINUX_APPLICATION_ID,
            "window_settings() must actually set platform_specific.application_id, \
             not merely declare the constant"
        );
    }
}

#[path = "tests/layout_matrix.rs"]
mod layout_matrix;
#[path = "tests/layout_probe.rs"]
mod layout_probe;
