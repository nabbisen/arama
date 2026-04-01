use std::path::Path;

pub fn is_hidden(path: &Path) -> bool {
    let start_with_dot = path
        .file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.starts_with('.'))
        .unwrap_or(false);

    if start_with_dot {
        return true;
    }

    // 2. Windows固有の属性チェック（必要であれば）
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if let Ok(metadata) = std::fs::metadata(path) {
            if (metadata.file_attributes() & 0x2) != 0 {
                return true;
            }
        }
    }

    false
}
