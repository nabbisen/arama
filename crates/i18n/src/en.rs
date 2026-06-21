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

        // Settings — AI tab
        "settings.ai.clip_missing" => {
            "AI model for image analysis is not found.\nShould get model from huggingface.co. Network will be used"
        }
        "settings.ai.clip_ready" => "AI model is ready.",
        "settings.ai.clip_load" => "Load",
        "settings.ai.clip_loading" => "loading...",
        "settings.ai.ffmpeg_missing" => {
            "ffmpeg for video analysis is not found.\nShould get executable. Network will be used"
        }
        "settings.ai.ffmpeg_ready" => "ffmpeg is ready.",
        "settings.ai.ffmpeg_get" => "Get",
        "settings.ai.ffmpeg_fetching" => "Downloading ffmpeg\u{2026}",

        // Settings — File system tab
        "settings.fs.cache_delete" => "Cache delete",

        // Settings — About tab
        "settings.about.repository" => "Repository:",

        // Cache page
        "cache.form.placeholder" => "/path/to/directory\u{2026}",
        "cache.form.button" => "Cache this dir",
        "cache.filter.placeholder" => "Filter by path\u{2026}",
        "cache.column.directory" => "Directory",
        "cache.column.files" => "Files",
        "cache.column.size" => "Size",
        "cache.column.cached_at" => "Cached at",
        "cache.row.caching" => "\u{23f3} caching\u{2026}",
        "cache.row.stop" => "Stop",
        "cache.empty" => "No cached directories yet.",
        "cache.no_match" => "No match.",
        "cache.summary.directories" => "directories",
        "cache.summary.files" => "files",
        "cache.summary.total" => "total",

        // Nav rail tooltips
        "nav.explorer" => "Explorer",
        "nav.cache" => "Cache",
        "nav.settings" => "Settings",

        // Setup wizard
        "setup.download" => "Download",
        "setup.skip" => "Skip",
        "setup.no_space" => "Not enough space on device for download.",
        "setup.item.clip" => "Image analysis AI model (CLIP)",
        "setup.item.wav2vec2" => "Audio analysis AI model (wav2vec2)",
        "setup.item.ffmpeg" => "Video manipulator (ffmpeg)",
        "setup.item.size_unknown" => "(unknown)",
        "setup.status.missing" => "Missing",
        "setup.status.downloading" => "Downloading...",
        "setup.status.ready" => "Ready",
        "setup.status.error" => "Error",
        "setup.not_ready" => "Not ready:",
        "setup.ready" => "Ready:",
        "setup.download_into" => "Will be downloaded into:",
        "setup.disk_space" => "Disk space",
        "setup.disk_gb_avail" => "GB available",
        "setup.disk_gb_total" => "GB total",

        // Focus dialog
        "focus.strategy" => "Cache lookup strategy",
        "focus.close" => "Close",

        // Similar-pairs dialog
        "pairs.no_valid" => "No valid pairs.",

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
            "High-contrast affects arama's own controls; some standard widgets use the base light/dark theme."
        }

        _ => return None,
    })
}
