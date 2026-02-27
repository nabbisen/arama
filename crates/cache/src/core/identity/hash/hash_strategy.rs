/// ファイル同一性確認の戦略。`CacheConfig` に組み込んで使う。
#[derive(Debug, Clone)]
pub enum HashStrategy {
    /// 常にファイル全体の SHA-256 を使う。
    /// 小ファイルや変更検出の精度を最優先する場合に適する。
    Full,

    /// ファイルサイズで Full と Partial を自動選択する (デフォルト)。
    ///
    /// - `threshold_bytes` 未満 → `Full` と同等
    /// - `threshold_bytes` 以上 → 先頭 + 末尾 `partial_bytes` の部分 SHA-256。
    ///   mtime をクイックフィルタとして DB に保存し、一致すれば Hash 計算を省略する。
    SizeAdaptive {
        /// 部分ハッシュへ切り替えるファイルサイズの閾値 (bytes)。
        threshold_bytes: u64,
        /// 先頭・末尾それぞれから読み取る bytes 数。
        partial_bytes: usize,
    },
}

impl Default for HashStrategy {
    fn default() -> Self {
        Self::SizeAdaptive {
            threshold_bytes: 4 * 1024 * 1024, // 4 MB
            partial_bytes: 64 * 1024,         // 64 KB × 2
        }
    }
}
