/// 計算済みフィンガープリント。DB への保存値と同形。
#[derive(Debug)]
pub struct FileFingerprint {
    /// SHA-256 の hex 文字列 (完全 or 部分)
    pub hash: String,
    /// `SizeAdaptive` で大ファイルと判定された場合のみ Some。
    /// `Full` または小ファイルでは None。
    pub mtime_ns: Option<i64>,
}
