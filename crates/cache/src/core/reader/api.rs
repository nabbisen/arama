//! 参照 API。
//!
//! | サブモジュール | 用途 |
//! |---|---|
//! | [`session`] | コネクションプールを保持する [`CacheReader`] ハンドル |
//! | [`oneshot`] | 単発呼び出し用 free function |

pub mod oneshot;
pub mod session;
