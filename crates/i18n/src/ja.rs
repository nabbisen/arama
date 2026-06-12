/// Look up `key` in the Japanese translation table.
pub(crate) fn get(key: &str) -> Option<&'static str> {
    Some(match key {
        // Settings — tabs
        "settings.tab.general"     => "\u{4e00}\u{822c}",          // 一般
        "settings.tab.ai"          => "AI",
        "settings.tab.filesystem"  => "\u{30d5}\u{30a1}\u{30a4}\u{30eb}\u{30b7}\u{30b9}\u{30c6}\u{30e0}",  // ファイルシステム
        "settings.tab.about"       => "\u{6982}\u{8981}",           // 概要

        // Settings — General tab
        "settings.general.include_image"     => "\u{753b}\u{50cf}",  // 画像
        "settings.general.include_video"     => "\u{52d5}\u{753b}",  // 動画
        "settings.general.sub_dir_depth"     => "\u{30b5}\u{30d6}\u{30c7}\u{30a3}\u{30ec}\u{30af}\u{30c8}\u{30ea}\u{6df1}\u{3055}",  // サブディレクトリ深さ
        "settings.general.similarity"        => "\u{985e}\u{4f3c}\u{5ea6}",  // 類似度
        "settings.general.language"          => "\u{8a00}\u{8a9e}",  // 言語

        // Settings — AI tab
        "settings.ai.clip_missing"  =>
            "\u{753b}\u{50cf}\u{89e3}\u{6790}\u{7528}AI\u{30e2}\u{30c7}\u{30eb}\u{304c}\u{898b}\u{3064}\u{304b}\u{308a}\u{307e}\u{305b}\u{3093}\u{3002}\nhuggingface.co\u{304b}\u{3089}\u{30e2}\u{30c7}\u{30eb}\u{3092}\u{53d6}\u{5f97}\u{3057}\u{307e}\u{3059}\u{3002}\u{30cd}\u{30c3}\u{30c8}\u{30ef}\u{30fc}\u{30af}\u{304c}\u{4f7f}\u{7528}\u{3055}\u{308c}\u{307e}\u{3059}",
        "settings.ai.clip_ready"    => "AI\u{30e2}\u{30c7}\u{30eb}\u{306f}\u{4f7f}\u{7528}\u{53ef}\u{80fd}\u{3067}\u{3059}\u{3002}",  // AIモデルは使用可能です。
        "settings.ai.clip_load"     => "\u{8aad}\u{307f}\u{8fbc}\u{307f}",  // 読み込み
        "settings.ai.clip_loading"  => "\u{8aad}\u{307f}\u{8fbc}\u{307f}\u{4e2d}...",  // 読み込み中...
        "settings.ai.ffmpeg_missing" =>
            "\u{52d5}\u{753b}\u{89e3}\u{6790}\u{7528}ffmpeg\u{304c}\u{898b}\u{3064}\u{304b}\u{308a}\u{307e}\u{305b}\u{3093}\u{3002}\n\u{5b9f}\u{884c}\u{30d5}\u{30a1}\u{30a4}\u{30eb}\u{3092}\u{53d6}\u{5f97}\u{3057}\u{307e}\u{3059}\u{3002}\u{30cd}\u{30c3}\u{30c8}\u{30ef}\u{30fc}\u{30af}\u{304c}\u{4f7f}\u{7528}\u{3055}\u{308c}\u{307e}\u{3059}",
        "settings.ai.ffmpeg_ready"     => "ffmpeg\u{306f}\u{4f7f}\u{7528}\u{53ef}\u{80fd}\u{3067}\u{3059}\u{3002}",  // ffmpegは使用可能です。
        "settings.ai.ffmpeg_get"       => "\u{53d6}\u{5f97}",  // 取得
        "settings.ai.ffmpeg_fetching"  => "ffmpeg\u{3092}\u{30c0}\u{30a6}\u{30f3}\u{30ed}\u{30fc}\u{30c9}\u{4e2d}\u{2026}",  // ffmpegをダウンロード中…

        // Settings — File system tab
        "settings.fs.cache_delete" => "\u{30ad}\u{30e3}\u{30c3}\u{30b7}\u{30e5}\u{524a}\u{9664}",  // キャッシュ削除

        // Settings — About tab
        "settings.about.repository" => "\u{30ea}\u{30dd}\u{30b8}\u{30c8}\u{30ea}\u{ff1a}",  // リポジトリ：

        // Cache page
        "cache.form.placeholder"   => "/path/to/directory\u{2026}",
        "cache.form.button"        => "\u{30c7}\u{30a3}\u{30ec}\u{30af}\u{30c8}\u{30ea}\u{3092}\u{30ad}\u{30e3}\u{30c3}\u{30b7}\u{30e5}",  // ディレクトリをキャッシュ
        "cache.filter.placeholder" => "\u{30d1}\u{30b9}\u{3067}\u{30d5}\u{30a3}\u{30eb}\u{30bf}\u{30fc}\u{2026}",  // パスでフィルター…
        "cache.column.directory"   => "\u{30c7}\u{30a3}\u{30ec}\u{30af}\u{30c8}\u{30ea}",  // ディレクトリ
        "cache.column.files"       => "\u{30d5}\u{30a1}\u{30a4}\u{30eb}\u{6570}",  // ファイル数
        "cache.column.size"        => "\u{30b5}\u{30a4}\u{30ba}",  // サイズ
        "cache.column.cached_at"   => "\u{30ad}\u{30e3}\u{30c3}\u{30b7}\u{30e5}\u{65e5}\u{6642}",  // キャッシュ日時
        "cache.row.caching"        => "\u{23f3} \u{30ad}\u{30e3}\u{30c3}\u{30b7}\u{30e5}\u{4e2d}\u{2026}",  // ⏳ キャッシュ中…
        "cache.row.stop"           => "\u{505c}\u{6b62}",  // 停止
        "cache.empty"              => "\u{30ad}\u{30e3}\u{30c3}\u{30b7}\u{30e5}\u{3055}\u{308c}\u{305f}\u{30c7}\u{30a3}\u{30ec}\u{30af}\u{30c8}\u{30ea}\u{306f}\u{3042}\u{308a}\u{307e}\u{305b}\u{3093}\u{3002}",  // キャッシュされたディレクトリはありません。
        "cache.no_match"           => "\u{4e00}\u{81f4}\u{306a}\u{3057}\u{3002}",  // 一致なし。
        "cache.summary.directories" => "\u{30c7}\u{30a3}\u{30ec}\u{30af}\u{30c8}\u{30ea}",  // ディレクトリ
        "cache.summary.files"       => "\u{30d5}\u{30a1}\u{30a4}\u{30eb}",  // ファイル
        "cache.summary.total"       => "\u{5408}\u{8a08}",  // 合計

        // Nav rail tooltips
        "nav.explorer" => "\u{30a8}\u{30af}\u{30b9}\u{30d7}\u{30ed}\u{30fc}\u{30e9}\u{30fc}",  // エクスプローラー
        "nav.cache"    => "\u{30ad}\u{30e3}\u{30c3}\u{30b7}\u{30e5}",  // キャッシュ
        "nav.settings" => "\u{8a2d}\u{5b9a}",  // 設定

        _ => return None,
    })
}
