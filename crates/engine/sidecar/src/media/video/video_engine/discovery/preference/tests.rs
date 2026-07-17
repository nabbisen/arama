use std::{cell::Cell, path::PathBuf};

use arama_env::{Settings, ffmpeg_location::FfmpegLocationPreference};

use super::{
    PreferenceRetainReason, PreferenceTransition, ValidatedSelection, clear_selection,
    prepare_selection, publish_validated_selection, reject_selection,
};
use crate::media::video::video_engine::{
    FfmpegToolchain, ToolchainSource,
    discovery::{DiscoverySource, FfmpegDiscoveryFailure, FfmpegDiscoveryOutcome, PairIssue},
};

fn absolute_directory(name: &str) -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(format!(r"C:\{name}"))
    } else {
        PathBuf::from(format!("/{name}"))
    }
}

fn selected(name: &str) -> FfmpegLocationPreference {
    FfmpegLocationPreference::SelectedDirectory(absolute_directory(name))
}

fn toolchain(directory: &std::path::Path) -> FfmpegToolchain {
    FfmpegToolchain {
        ffmpeg: directory.join(if cfg!(windows) {
            "ffmpeg.exe"
        } else {
            "ffmpeg"
        }),
        ffprobe: directory.join(if cfg!(windows) {
            "ffprobe.exe"
        } else {
            "ffprobe"
        }),
        source: ToolchainSource::System,
    }
}

#[test]
fn validation_for_directory_a_cannot_be_bound_to_directory_b() {
    let current = FfmpegLocationPreference::Auto;
    let candidate_a = prepare_selection(&current, Some(absolute_directory("a"))).unwrap();
    let candidate_b = prepare_selection(&current, Some(absolute_directory("b"))).unwrap();
    let authority_a = toolchain(&absolute_directory("a"));

    assert!(ValidatedSelection::bind(candidate_b, authority_a.clone()).is_err());
    let validated = ValidatedSelection::bind(candidate_a, authority_a).unwrap();
    assert_eq!(validated.preference, selected("a"));
}

#[cfg(unix)]
#[test]
fn selected_symlink_binds_when_tool_paths_keep_the_selected_lexical_parent() {
    use std::{fs, os::unix::fs::symlink};

    let root = std::env::current_dir()
        .unwrap()
        .join("target/selected-symlink-tests")
        .join(std::process::id().to_string());
    let real = root.join("real");
    let selected_alias = root.join("selected-alias");
    fs::create_dir_all(&real).unwrap();
    symlink(&real, &selected_alias).unwrap();
    let current = FfmpegLocationPreference::Auto;
    let candidate = prepare_selection(&current, Some(selected_alias.clone())).unwrap();

    let validated = ValidatedSelection::bind(candidate, toolchain(&selected_alias)).unwrap();

    assert_eq!(
        validated.preference,
        FfmpegLocationPreference::SelectedDirectory(selected_alias)
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn successful_full_settings_save_publishes_exact_ready_authority() {
    let current = FfmpegLocationPreference::Auto;
    let candidate = prepare_selection(&current, Some(absolute_directory("new/bin"))).unwrap();
    let expected_toolchain = toolchain(&absolute_directory("new/bin"));
    let validated = ValidatedSelection::bind(candidate, expected_toolchain.clone()).unwrap();
    let mut settings = Settings {
        root_dir_path: "preserved-root".to_owned(),
        ..Settings::default()
    };

    let transition = publish_validated_selection(
        &mut settings,
        &current,
        validated,
        |saved_settings| -> Result<(), ()> {
            assert_eq!(saved_settings.root_dir_path, "preserved-root");
            assert_eq!(saved_settings.ffmpeg_location, selected("new/bin"));
            Ok(())
        },
    );

    assert_eq!(
        transition,
        PreferenceTransition::PublishedReady {
            preference: selected("new/bin"),
            outcome: FfmpegDiscoveryOutcome::Ready {
                toolchain: expected_toolchain,
                source: super::DiscoverySource::SelectedDirectory,
            },
        }
    );
    assert_eq!(settings.ffmpeg_location, selected("new/bin"));
}

#[test]
fn mismatched_or_failed_save_cannot_publish_candidate() {
    let current = selected("old/bin");
    let candidate = prepare_selection(&current, Some(absolute_directory("new/bin"))).unwrap();
    let validated =
        ValidatedSelection::bind(candidate, toolchain(&absolute_directory("new/bin"))).unwrap();
    let mut mismatched_settings = Settings::default();
    let save_called = Cell::new(false);

    let transition = publish_validated_selection(
        &mut mismatched_settings,
        &current,
        validated,
        |_| -> Result<(), ()> {
            save_called.set(true);
            Ok(())
        },
    );
    assert!(!save_called.get());
    assert_eq!(
        transition,
        PreferenceTransition::Retained {
            preference: current.clone(),
            reason: PreferenceRetainReason::SettingsAuthorityMismatch,
            candidate_outcome: None,
        }
    );

    let candidate = prepare_selection(&current, Some(absolute_directory("new/bin"))).unwrap();
    let validated =
        ValidatedSelection::bind(candidate, toolchain(&absolute_directory("new/bin"))).unwrap();
    let mut settings = Settings {
        ffmpeg_location: current.clone(),
        ..Settings::default()
    };
    let transition =
        publish_validated_selection(&mut settings, &current, validated, |_| -> Result<(), ()> {
            Err(())
        });
    assert_eq!(settings.ffmpeg_location, current);
    assert!(matches!(
        transition,
        PreferenceTransition::Retained {
            reason: PreferenceRetainReason::SaveFailure,
            ..
        }
    ));
}

#[test]
fn failed_typed_validation_preserves_prior_authority_and_reason() {
    let current = selected("old/bin");
    let candidate = prepare_selection(&current, Some(absolute_directory("bad/bin"))).unwrap();
    let outcome = FfmpegDiscoveryOutcome::InvalidPair(PairIssue::VersionMismatch);
    let failure = FfmpegDiscoveryFailure::try_from(outcome.clone()).unwrap();

    assert_eq!(
        reject_selection(&current, candidate, failure),
        PreferenceTransition::Retained {
            preference: current,
            reason: PreferenceRetainReason::InvalidSelection,
            candidate_outcome: Some(outcome),
        }
    );
}

#[test]
fn ready_authority_cannot_be_represented_as_a_rejection() {
    let ready = FfmpegDiscoveryOutcome::Ready {
        toolchain: toolchain(&absolute_directory("ready/bin")),
        source: DiscoverySource::SelectedDirectory,
    };

    assert_eq!(FfmpegDiscoveryFailure::try_from(ready.clone()), Err(ready));
}

#[test]
fn cancelled_and_relative_selection_retain_prior_authority() {
    let current = selected("old/bin");
    for transition in [
        prepare_selection(&current, None).unwrap_err(),
        prepare_selection(&current, Some(PathBuf::from("relative/bin"))).unwrap_err(),
    ] {
        let PreferenceTransition::Retained { preference, .. } = transition else {
            panic!("prior authority should be retained");
        };
        assert_eq!(preference, current);
    }
}

#[test]
fn clear_saves_exact_auto_settings_before_publication() {
    let current = selected("old/bin");
    let mut settings = Settings {
        ffmpeg_location: current.clone(),
        ..Settings::default()
    };
    let transition = clear_selection(&mut settings, &current, |saved| -> Result<(), ()> {
        assert_eq!(saved.ffmpeg_location, FfmpegLocationPreference::Auto);
        Ok(())
    });

    assert_eq!(transition, PreferenceTransition::PublishedAuto);
    assert_eq!(settings.ffmpeg_location, FfmpegLocationPreference::Auto);
}
