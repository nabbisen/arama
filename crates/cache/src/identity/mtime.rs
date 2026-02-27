use std::time::SystemTime;

pub fn read_mtime(meta: &std::fs::Metadata) -> Option<i64> {
    let t = meta.modified().ok()?;
    match t.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => i64::try_from(d.as_nanos()).ok(),
        // 1970 以前のタイムスタンプは符号付きで保存 (実用上ほぼ発生しない)
        Err(e) => i64::try_from(e.duration().as_nanos()).ok().map(|n| -n),
    }
}
