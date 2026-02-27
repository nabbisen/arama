//! ファイル同一性確認ロジック。
//!
//! # 戦略の選択基準
//!
//! | ファイルサイズ | ハッシュ種別 | mtime の扱い |
//! |---|---|---|
//! | 閾値未満 (デフォルト 4 MB) | 完全 SHA-256 | 保存しない |
//! | 閾値以上 | 先頭 + 末尾の部分 SHA-256 | クイックフィルタとして保存 |
//!
//! ## 大ファイルの lookup フロー
//!
//! ```text
//! stored_mtime == current_mtime ──→ Hash 計算スキップ → Hit fast path
//!                ↓ 不一致
//!          部分 Hash を再計算
//!          stored_hash == new_hash ──→ Hit (内容は同じ、mtime だけ更新)
//!                         ↓ 不一致
//!                      Invalidated
//! ```

pub mod api;
mod file_fingerprint;
pub mod hash_strategy;
mod mode;
mod util;
