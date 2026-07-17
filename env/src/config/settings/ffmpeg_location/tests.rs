use std::path::PathBuf;

use super::{FfmpegLocationPreference, FfmpegPreferenceError};

fn absolute_directory(name: &str) -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(format!(r"C:\{name}"))
    } else {
        PathBuf::from(format!("/{name}"))
    }
}

#[test]
fn default_and_missing_setting_are_auto() {
    assert_eq!(
        FfmpegLocationPreference::default(),
        FfmpegLocationPreference::Auto
    );

    let mut encoded = serde_json::to_value(crate::Settings::default()).unwrap();
    encoded.as_object_mut().unwrap().remove("ffmpeg_location");
    let settings: crate::Settings =
        serde_json::from_value(encoded).expect("older settings should deserialize");
    assert_eq!(settings.ffmpeg_location, FfmpegLocationPreference::Auto);
}

#[test]
fn selected_absolute_directory_round_trips() {
    let preference = FfmpegLocationPreference::SelectedDirectory(absolute_directory("ffmpeg-bin"));
    preference.validate_persistable().unwrap();
    let encoded = serde_json::to_string(&preference).unwrap();
    let decoded = serde_json::from_str(&encoded).unwrap();
    assert_eq!(preference, decoded);
}

#[test]
fn relative_selection_is_rejected() {
    let preference = FfmpegLocationPreference::SelectedDirectory(PathBuf::from("tools/bin"));
    assert_eq!(
        preference.validate_persistable(),
        Err(FfmpegPreferenceError::EmptyOrRelative)
    );
}

#[cfg(unix)]
#[test]
fn non_unicode_selection_is_rejected_before_json_save() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let mut path = PathBuf::from("/");
    path.push(OsString::from_vec(vec![0xff]));
    let preference = FfmpegLocationPreference::SelectedDirectory(path);
    assert_eq!(
        preference.validate_persistable(),
        Err(FfmpegPreferenceError::NonUnicode)
    );
}
