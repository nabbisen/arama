/// Look up `key` in the English translation table.
pub(crate) fn get(key: &str) -> Option<&'static str> {
    Some(match key {
        // Settings — tabs
        "settings.tab.general" => "General",
        "settings.tab.ai" => "AI",
        "settings.tab.filesystem" => "File system",
        "settings.tab.about" => "About",

        // Settings — General tab
        "settings.general.include_image" => "Image",
        "settings.general.include_video" => "Video",
        "settings.general.sub_dir_depth" => "Sub dir depth",
        "settings.general.similarity" => "Similarity",
        "settings.general.language" => "Language",
        "settings.load_error.title" => "Settings load failed",
        "settings.load_error.body" => "Using default settings for this session",
        "settings.save_error.title" => "Settings save failed",

        // Settings — AI tab
        "settings.ai.clip_missing" => {
            "AI model for image analysis is not found.\nShould get model from huggingface.co. Network will be used"
        }
        "settings.ai.clip_ready" => "AI model is ready.",
        "settings.ai.clip_load" => "Load",
        "settings.ai.clip_loading" => "loading...",
        "settings.ai.wav2vec2_missing" => {
            "Optional audio-analysis model (Wav2Vec2) is not installed."
        }
        "settings.ai.wav2vec2_ready" => "Optional Wav2Vec2 model is ready.",
        "settings.ai.wav2vec2_get" => "Download Wav2Vec2",
        "settings.ai.wav2vec2_downloading" => "Downloading Wav2Vec2\u{2026}",
        "settings.ai.wav2vec2_error" => "Wav2Vec2 download failed",
        "settings.ai.wav2vec2_retry" => "Retry Wav2Vec2 download",
        "settings.ai.ffmpeg_ready" => "ffmpeg is ready.",
        "settings.ai.ffmpeg_checking" => "Checking for a valid ffmpeg and ffprobe pair\u{2026}",
        "settings.ai.ffmpeg_draining" => {
            "A previous FFmpeg check is still stopping. The latest check will start automatically."
        }
        "settings.ai.ffmpeg_external" => {
            "A matching ffmpeg and ffprobe pair is required for video analysis. Install it through a trusted source for your platform, then re-check."
        }
        "settings.ai.ffmpeg_recheck" => "Re-check",
        "settings.ai.ffmpeg_select" => "Select ffmpeg folder",
        "settings.ai.ffmpeg_clear" => "Use automatic discovery",
        "settings.ai.ffmpeg_mode_auto" => "Discovery mode: automatic",
        "settings.ai.ffmpeg_mode_selected" => "Selected folder",
        "settings.ai.ffmpeg_candidate_rejected" => "Selected folder was not accepted",
        "settings.ai.ffmpeg_candidate_checking" => "Checking selected folder",
        "settings.ai.ffmpeg_invalid_pair" => {
            "The folder must contain a matching ffmpeg and ffprobe executable pair."
        }
        "settings.ai.ffmpeg_probe_timed_out" => {
            "The executable version check timed out. Check the selected binaries and try again."
        }
        "settings.ai.ffmpeg_search_limited" => {
            "The bounded FFmpeg search could not finish. Reduce the search scope or try again."
        }
        "settings.ai.ffmpeg_legacy_excluded" => {
            "This folder is arama's old managed FFmpeg location. Install or move the pair outside arama, then select that folder."
        }
        "settings.ai.ffmpeg_invalid_path" => "Select a valid absolute folder path.",
        "settings.ai.ffmpeg_filesystem_unavailable_auto" => {
            "arama could not check part of your PATH due to a filesystem error. Select the ffmpeg folder directly, or try again."
        }
        "settings.ai.ffmpeg_filesystem_unavailable_selected" => {
            "arama could not verify this folder's filesystem identity or access. Check permissions and try again."
        }

        // Settings — File system tab
        "settings.fs.cache_delete" => "Cache delete",
        "settings.fs.disk_unavailable" => "Disk space unavailable",

        // Settings — About tab
        "settings.about.repository" => "Repository:",

        // Cache page
        "cache.form.placeholder" => "/path/to/directory\u{2026}",
        "cache.form.button" => "Cache this dir",
        "cache.filter.placeholder" => "Filter by path\u{2026}",
        "cache.column.directory" => "Directory",
        "cache.column.files" => "Files",
        "cache.column.size" => "Media size",
        "cache.column.cached_at" => "Cached at",
        "cache.footprint" => "Cache footprint",
        "cache.footprint.unavailable" => "unavailable",
        "cache.load_error" => "Cache page reload failed",
        "cache.load_error.stale" => "Cache page reload failed; showing stale data",
        "cache.prune.placeholder" => "Target",
        "cache.prune.unit_mib" => "MiB",
        "cache.prune.button" => "Prune",
        "cache.prune.done" => "Pruned",
        "cache.prune.partial" => "Pruned; target not reached",
        "cache.prune.entries" => "entries",
        "cache.prune.unreclaimable" => "unreclaimable",
        "cache.row.caching" => "\u{23f3} caching\u{2026}",
        "cache.row.stop" => "Stop",
        "cache.empty" => "No cached directories yet.",
        "cache.no_match" => "No match.",
        "cache.summary.directories" => "directories",
        "cache.summary.files" => "files",
        "cache.summary.total" => "media total",
        "cache.time.just_now" => "just now",
        "cache.time.ago" => "ago",
        "cache.time.minute" => "minute",
        "cache.time.minutes" => "minutes",
        "cache.time.hour" => "hour",
        "cache.time.hours" => "hours",
        "cache.time.day" => "day",
        "cache.time.days" => "days",
        "cache.time.month" => "month",
        "cache.time.months" => "months",
        "cache.time.year" => "year",
        "cache.time.years" => "years",

        // Aside tree toggle
        "aside.toggle.open" => "Open folder tree",
        "aside.toggle.close" => "Close folder tree",
        // Nav rail tooltips
        "nav.explorer" => "Explorer",
        "nav.cache" => "Cache",
        "nav.settings" => "Settings",

        // Footer (RFC 044 §3.1)
        "footer.f6_hint" => "F6 switches panels",
        // Footer (Task 031)
        "footer.thumbnail_size" => "Thumbnail size",
        "footer.files_count" => "files",
        "footer.dirs_scanned" => "dirs scanned",

        // Setup wizard
        "setup.download" => "Download",
        "setup.skip" => "Skip",
        "setup.no_space" => "Not enough space on device for download.",
        "setup.item.clip" => "Image analysis AI model (CLIP)",
        "setup.item.wav2vec2" => "Audio analysis AI model (wav2vec2)",
        "setup.item.ffmpeg" => "Video manipulator (ffmpeg)",
        "setup.item.size_unknown" => "(unknown)",
        "setup.status.missing" => "Missing",
        "setup.status.checking" => "Checking\u{2026}",
        "setup.status.ffmpeg_worker_draining" => {
            "Waiting for the previous FFmpeg check to stop\u{2026}"
        }
        "setup.status.external_required" => "External installation required",
        "setup.status.downloading" => "Downloading...",
        "setup.status.ready" => "Ready",
        "setup.status.error" => "Error",
        "setup.not_ready" => "Not ready:",
        "setup.ready" => "Ready:",
        "setup.download_into" => "AI models will be downloaded into:",
        "setup.disk_space" => "Disk space",
        "setup.disk_gb_avail" => "GB available",
        "setup.disk_gb_total" => "GB total",
        "setup.ffmpeg.external_help" => {
            "Install a matching ffmpeg and ffprobe pair through a trusted source for your platform, then re-check. Image-only use can continue without it."
        }
        "setup.ffmpeg.recheck" => "Re-check",
        "setup.ffmpeg.select" => "Select ffmpeg folder",

        // Similarity dialogs (shared: similar-pairs and focus)
        "similarity.read_error" => "Some files could not be read; results may be incomplete.",
        "similarity.nothing_indexed" => "Nothing has been indexed yet.",
        "similarity.no_results" => "No similar items found.",
        "similarity.video_unavailable" => {
            "Video comparison did not run: no ffmpeg/ffprobe pair was found."
        }

        // Focus dialog
        "focus.strategy" => "Cache lookup strategy",
        "focus.close" => "Close",

        // Header
        "header.folder" => "Folder",

        // Gallery
        "gallery.empty" => "No file to render.",

        // Gallery filter
        "gallery.filter.placeholder" => "Filter by filename…",
        "gallery.filter.clear" => "✕",
        "gallery.filter.count_of" => "of",

        // Theme selector (RFC 011)
        "settings.general.theme" => "Theme",
        "settings.general.theme.light" => "Light",
        "settings.general.theme.dark" => "Dark",
        "settings.general.theme.hc_light" => "High contrast light",
        "settings.general.theme.hc_dark" => "High contrast dark",
        "settings.general.theme.hc_note" => {
            "High-contrast maps core colors into standard widgets; arama controls use the full high-contrast palette."
        }

        // Startup
        "startup.fatal_error.title" => "arama could not start",
        "startup.fatal_error.body" => {
            "arama could not create the location it needs to store its settings, models, or \
             cache, and has nowhere to save anything. Check that arama has permission to write \
             to its data directory, then restart."
        }
        "startup.root_dir_unavailable.title" => "Startup folder unavailable",
        "startup.root_dir_unavailable.body" => {
            "The saved folder cannot be opened. Choose another folder to start indexing"
        }
        "startup.root_scan_warning.title" => "Folder scan incomplete",
        "startup.root_scan_warning.body" => "Some folders could not be read during startup",

        _ => return None,
    })
}
