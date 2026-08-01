use std::{path::PathBuf, process::Stdio};

use arama_env::{Settings, ffmpeg_location::FfmpegLocationPreference};
use arama_sidecar::media::video::video_engine::discovery::{
    FfmpegDiscoveryEvent, FfmpegDiscoveryOutcome, FfmpegDiscoveryRuntime, PreferenceTransition,
    publish_validated_selection,
};

#[test]
#[ignore = "owner-run native smoke; set ARAMA_FFMPEG_SMOKE_DIR to a trusted pair"]
fn selected_external_pair_generates_probes_and_extracts_real_video() {
    let directory = std::env::var_os("ARAMA_FFMPEG_SMOKE_DIR")
        .map(PathBuf::from)
        .expect("set ARAMA_FFMPEG_SMOKE_DIR to the directory containing ffmpeg and ffprobe");
    assert!(directory.is_absolute());

    let preference = FfmpegLocationPreference::SelectedDirectory(directory);
    let runtime = FfmpegDiscoveryRuntime::default();
    let ticket = runtime.request(preference.clone());
    let async_runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("build smoke runtime");

    assert!(matches!(
        async_runtime.block_on(ticket.next()),
        Some(FfmpegDiscoveryEvent::Started(1))
    ));
    let Some(FfmpegDiscoveryEvent::SelectedReady { validated, .. }) =
        async_runtime.block_on(ticket.next())
    else {
        panic!("selected external pair did not validate");
    };

    let mut settings = Settings {
        ffmpeg_location: FfmpegLocationPreference::Auto,
        ..Settings::default()
    };
    let PreferenceTransition::PublishedReady { outcome, .. } = publish_validated_selection(
        &mut settings,
        &FfmpegLocationPreference::Auto,
        validated,
        |_| Ok::<(), ()>(()),
    ) else {
        panic!("validated pair did not publish");
    };
    let FfmpegDiscoveryOutcome::Ready { toolchain, .. } = outcome else {
        panic!("published selection did not carry toolchain authority");
    };

    let root = std::env::current_dir()
        .expect("current directory")
        .join("target/external-ffmpeg-smoke")
        .join(std::process::id().to_string());
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create smoke directory");
    let video = root.join("fixture.mp4");
    let frame = root.join("frame.png");

    let generated = toolchain
        .ffmpeg()
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "color=c=blue:s=64x64:d=1",
            "-c:v",
            "mpeg4",
            "-y",
        ])
        .arg(&video)
        .stdin(Stdio::null())
        .status()
        .expect("run captured ffmpeg for fixture generation");
    assert!(generated.success());

    let probe = toolchain
        .ffprobe()
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(&video)
        .stdin(Stdio::null())
        .output()
        .expect("run captured ffprobe");
    assert!(probe.status.success());
    let duration: f64 = String::from_utf8(probe.stdout)
        .expect("duration is UTF-8")
        .trim()
        .parse()
        .expect("duration is numeric");
    assert!(duration > 0.0);

    let extracted = toolchain
        .ffmpeg()
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(&video)
        .args(["-frames:v", "1", "-y"])
        .arg(&frame)
        .stdin(Stdio::null())
        .status()
        .expect("run captured ffmpeg for frame extraction");
    assert!(extracted.success());
    assert!(std::fs::metadata(&frame).is_ok_and(|metadata| metadata.len() > 0));

    std::fs::remove_dir_all(root).expect("remove smoke directory");
}
