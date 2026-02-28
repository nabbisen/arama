//! 更新 API。
//!
//! | サブモジュール | 用途 |
//! |---|---|
//! | [`session`] | コネクションプールを保持する [`CacheWriter`] ハンドル |
//! | [`oneshot`] | 単発呼び出し用 free function |

pub mod oneshot;
pub mod session;
