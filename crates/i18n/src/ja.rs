/// Look up `key` in the Japanese translation table.
pub(crate) fn get(key: &str) -> Option<&'static str> {
    Some(match key {
        // Settings — tabs
        "settings.tab.general" => "\u{4e00}\u{822c}", // 一般
        "settings.tab.ai" => "AI",
        "settings.tab.filesystem" => {
            "\u{30d5}\u{30a1}\u{30a4}\u{30eb}\u{30b7}\u{30b9}\u{30c6}\u{30e0}"
        } // ファイルシステム
        "settings.tab.about" => "\u{6982}\u{8981}", // 概要

        // Settings — General tab
        "settings.general.include_image" => "\u{753b}\u{50cf}", // 画像
        "settings.general.include_video" => "\u{52d5}\u{753b}", // 動画
        "settings.general.sub_dir_depth" => {
            "\u{30b5}\u{30d6}\u{30c7}\u{30a3}\u{30ec}\u{30af}\u{30c8}\u{30ea}\u{6df1}\u{3055}"
        } // サブディレクトリ深さ
        "settings.general.similarity" => "\u{985e}\u{4f3c}\u{5ea6}", // 類似度
        "settings.general.language" => "\u{8a00}\u{8a9e}",      // 言語
        "settings.load_error.title" => {
            "\u{8a2d}\u{5b9a}\u{306e}\u{8aad}\u{307f}\u{8fbc}\u{307f}\u{306b}\u{5931}\u{6557}\u{3057}\u{307e}\u{3057}\u{305f}"
        } // 設定の読み込みに失敗しました
        "settings.load_error.body" => {
            "\u{3053}\u{306e}\u{30bb}\u{30c3}\u{30b7}\u{30e7}\u{30f3}\u{3067}\u{306f}\u{65e2}\u{5b9a}\u{306e}\u{8a2d}\u{5b9a}\u{3092}\u{4f7f}\u{7528}\u{3057}\u{307e}\u{3059}"
        } // このセッションでは既定の設定を使用します
        "settings.save_error.title" => {
            "\u{8a2d}\u{5b9a}\u{306e}\u{4fdd}\u{5b58}\u{306b}\u{5931}\u{6557}\u{3057}\u{307e}\u{3057}\u{305f}"
        } // 設定の保存に失敗しました

        // Settings — AI tab
        "settings.ai.clip_missing" => {
            "\u{753b}\u{50cf}\u{89e3}\u{6790}\u{7528}AI\u{30e2}\u{30c7}\u{30eb}\u{304c}\u{898b}\u{3064}\u{304b}\u{308a}\u{307e}\u{305b}\u{3093}\u{3002}\nhuggingface.co\u{304b}\u{3089}\u{30e2}\u{30c7}\u{30eb}\u{3092}\u{53d6}\u{5f97}\u{3057}\u{307e}\u{3059}\u{3002}\u{30cd}\u{30c3}\u{30c8}\u{30ef}\u{30fc}\u{30af}\u{304c}\u{4f7f}\u{7528}\u{3055}\u{308c}\u{307e}\u{3059}"
        }
        "settings.ai.clip_ready" => {
            "AI\u{30e2}\u{30c7}\u{30eb}\u{306f}\u{4f7f}\u{7528}\u{53ef}\u{80fd}\u{3067}\u{3059}\u{3002}"
        } // AIモデルは使用可能です。
        "settings.ai.clip_load" => "\u{8aad}\u{307f}\u{8fbc}\u{307f}", // 読み込み
        "settings.ai.clip_loading" => "\u{8aad}\u{307f}\u{8fbc}\u{307f}\u{4e2d}...", // 読み込み中...
        "settings.ai.wav2vec2_missing" => {
            "\u{4efb}\u{610f}\u{306e}\u{97f3}\u{58f0}\u{89e3}\u{6790}\u{30e2}\u{30c7}\u{30eb}\u{ff08}Wav2Vec2\u{ff09}\u{304c}\u{30a4}\u{30f3}\u{30b9}\u{30c8}\u{30fc}\u{30eb}\u{3055}\u{308c}\u{3066}\u{3044}\u{307e}\u{305b}\u{3093}\u{3002}"
        }
        "settings.ai.wav2vec2_ready" => {
            "\u{4efb}\u{610f}\u{306e}Wav2Vec2\u{30e2}\u{30c7}\u{30eb}\u{306f}\u{4f7f}\u{7528}\u{53ef}\u{80fd}\u{3067}\u{3059}\u{3002}"
        }
        "settings.ai.wav2vec2_get" => {
            "Wav2Vec2\u{3092}\u{30c0}\u{30a6}\u{30f3}\u{30ed}\u{30fc}\u{30c9}"
        }
        "settings.ai.wav2vec2_downloading" => {
            "Wav2Vec2\u{3092}\u{30c0}\u{30a6}\u{30f3}\u{30ed}\u{30fc}\u{30c9}\u{4e2d}\u{2026}"
        }
        "settings.ai.wav2vec2_error" => {
            "Wav2Vec2\u{306e}\u{30c0}\u{30a6}\u{30f3}\u{30ed}\u{30fc}\u{30c9}\u{306b}\u{5931}\u{6557}\u{3057}\u{307e}\u{3057}\u{305f}"
        }
        "settings.ai.wav2vec2_retry" => {
            "Wav2Vec2\u{306e}\u{30c0}\u{30a6}\u{30f3}\u{30ed}\u{30fc}\u{30c9}\u{3092}\u{518d}\u{8a66}\u{884c}"
        }
        "settings.ai.ffmpeg_ready" => {
            "ffmpeg\u{306f}\u{4f7f}\u{7528}\u{53ef}\u{80fd}\u{3067}\u{3059}\u{3002}"
        } // ffmpegは使用可能です。
        "settings.ai.ffmpeg_checking" => {
            "ffmpeg\u{3068}ffprobe\u{306e}\u{6709}\u{52b9}\u{306a}\u{30da}\u{30a2}\u{3092}\u{78ba}\u{8a8d}\u{4e2d}\u{2026}"
        }
        "settings.ai.ffmpeg_draining" => {
            "\u{524d}\u{306e}FFmpeg\u{78ba}\u{8a8d}\u{306e}\u{505c}\u{6b62}\u{3092}\u{5f85}\u{3063}\u{3066}\u{3044}\u{307e}\u{3059}\u{3002}\u{6700}\u{65b0}\u{306e}\u{78ba}\u{8a8d}\u{306f}\u{81ea}\u{52d5}\u{7684}\u{306b}\u{958b}\u{59cb}\u{3055}\u{308c}\u{307e}\u{3059}\u{3002}"
        }
        "settings.ai.ffmpeg_external" => {
            "\u{52d5}\u{753b}\u{89e3}\u{6790}\u{306b}\u{306f}\u{540c}\u{3058}\u{30d0}\u{30fc}\u{30b8}\u{30e7}\u{30f3}\u{306e}ffmpeg\u{3068}ffprobe\u{304c}\u{5fc5}\u{8981}\u{3067}\u{3059}\u{3002}\u{304a}\u{4f7f}\u{3044}\u{306e}\u{30d7}\u{30e9}\u{30c3}\u{30c8}\u{30d5}\u{30a9}\u{30fc}\u{30e0}\u{7528}\u{306e}\u{4fe1}\u{983c}\u{3067}\u{304d}\u{308b}\u{914d}\u{5e03}\u{5143}\u{304b}\u{3089}\u{30a4}\u{30f3}\u{30b9}\u{30c8}\u{30fc}\u{30eb}\u{3057}\u{3001}\u{518d}\u{78ba}\u{8a8d}\u{3057}\u{3066}\u{304f}\u{3060}\u{3055}\u{3044}\u{3002}"
        }
        "settings.ai.ffmpeg_recheck" => "\u{518d}\u{78ba}\u{8a8d}",
        "settings.ai.ffmpeg_select" => {
            "ffmpeg \u{30d5}\u{30a9}\u{30eb}\u{30c0}\u{30fc}\u{3092}\u{9078}\u{629e}"
        }
        "settings.ai.ffmpeg_clear" => "\u{81ea}\u{52d5}\u{691c}\u{51fa}\u{3092}\u{4f7f}\u{3046}",
        "settings.ai.ffmpeg_mode_auto" => {
            "\u{691c}\u{51fa}\u{30e2}\u{30fc}\u{30c9}: \u{81ea}\u{52d5}"
        }
        "settings.ai.ffmpeg_mode_selected" => {
            "\u{9078}\u{629e}\u{4e2d}\u{306e}\u{30d5}\u{30a9}\u{30eb}\u{30c0}\u{30fc}"
        }
        "settings.ai.ffmpeg_candidate_rejected" => {
            "\u{9078}\u{629e}\u{3057}\u{305f}\u{30d5}\u{30a9}\u{30eb}\u{30c0}\u{30fc}\u{306f}\u{4f7f}\u{7528}\u{3067}\u{304d}\u{307e}\u{305b}\u{3093}"
        }
        "settings.ai.ffmpeg_candidate_checking" => {
            "\u{9078}\u{629e}\u{3057}\u{305f}\u{30d5}\u{30a9}\u{30eb}\u{30c0}\u{30fc}\u{3092}\u{78ba}\u{8a8d}\u{4e2d}"
        }
        "settings.ai.ffmpeg_invalid_pair" => {
            "\u{540c}\u{3058}\u{30d0}\u{30fc}\u{30b8}\u{30e7}\u{30f3}\u{306e}ffmpeg\u{3068}ffprobe\u{306e}\u{5b9f}\u{884c}\u{30d5}\u{30a1}\u{30a4}\u{30eb}\u{3092}\u{3053}\u{306e}\u{30d5}\u{30a9}\u{30eb}\u{30c0}\u{30fc}\u{306b}\u{7f6e}\u{3044}\u{3066}\u{304f}\u{3060}\u{3055}\u{3044}\u{3002}"
        }
        "settings.ai.ffmpeg_probe_timed_out" => {
            "\u{5b9f}\u{884c}\u{30d5}\u{30a1}\u{30a4}\u{30eb}\u{306e}\u{30d0}\u{30fc}\u{30b8}\u{30e7}\u{30f3}\u{78ba}\u{8a8d}\u{304c}\u{30bf}\u{30a4}\u{30e0}\u{30a2}\u{30a6}\u{30c8}\u{3057}\u{307e}\u{3057}\u{305f}\u{3002}"
        }
        "settings.ai.ffmpeg_search_limited" => {
            "FFmpeg\u{306e}\u{691c}\u{7d22}\u{3092}\u{5236}\u{9650}\u{6642}\u{9593}\u{5185}\u{306b}\u{5b8c}\u{4e86}\u{3067}\u{304d}\u{307e}\u{305b}\u{3093}\u{3067}\u{3057}\u{305f}\u{3002}"
        }
        "settings.ai.ffmpeg_legacy_excluded" => {
            "\u{3053}\u{306e}\u{30d5}\u{30a9}\u{30eb}\u{30c0}\u{30fc}\u{306f}arama\u{306e}\u{65e7}\u{7ba1}\u{7406}\u{5834}\u{6240}\u{3067}\u{3059}\u{3002}arama\u{306e}\u{5916}\u{306b}\u{30da}\u{30a2}\u{3092}\u{8a2d}\u{7f6e}\u{3057}\u{3001}\u{305d}\u{306e}\u{30d5}\u{30a9}\u{30eb}\u{30c0}\u{30fc}\u{3092}\u{9078}\u{629e}\u{3057}\u{3066}\u{304f}\u{3060}\u{3055}\u{3044}\u{3002}"
        }
        "settings.ai.ffmpeg_invalid_path" => {
            "\u{6709}\u{52b9}\u{306a}\u{7d76}\u{5bfe}\u{30d1}\u{30b9}\u{306e}\u{30d5}\u{30a9}\u{30eb}\u{30c0}\u{30fc}\u{3092}\u{9078}\u{629e}\u{3057}\u{3066}\u{304f}\u{3060}\u{3055}\u{3044}\u{3002}"
        }
        "settings.ai.ffmpeg_filesystem_unavailable" => {
            "\u{30d5}\u{30a1}\u{30a4}\u{30eb}\u{30b7}\u{30b9}\u{30c6}\u{30e0}\u{306e}\u{8b58}\u{5225}\u{60c5}\u{5831}\u{307e}\u{305f}\u{306f}\u{30a2}\u{30af}\u{30bb}\u{30b9}\u{6a29}\u{3092}\u{78ba}\u{8a8d}\u{3067}\u{304d}\u{307e}\u{305b}\u{3093}\u{3067}\u{3057}\u{305f}\u{3002}"
        }

        // Settings — File system tab
        "settings.fs.cache_delete" => "\u{30ad}\u{30e3}\u{30c3}\u{30b7}\u{30e5}\u{524a}\u{9664}", // キャッシュ削除
        "settings.fs.disk_unavailable" => {
            "\u{30c7}\u{30a3}\u{30b9}\u{30af}\u{5bb9}\u{91cf}\u{3092}\u{53d6}\u{5f97}\u{3067}\u{304d}\u{307e}\u{305b}\u{3093}"
        } // ディスク容量を取得できません

        // Settings — About tab
        "settings.about.repository" => "\u{30ea}\u{30dd}\u{30b8}\u{30c8}\u{30ea}\u{ff1a}", // リポジトリ：

        // Cache page
        "cache.form.placeholder" => "/path/to/directory\u{2026}",
        "cache.form.button" => {
            "\u{30c7}\u{30a3}\u{30ec}\u{30af}\u{30c8}\u{30ea}\u{3092}\u{30ad}\u{30e3}\u{30c3}\u{30b7}\u{30e5}"
        } // ディレクトリをキャッシュ
        "cache.filter.placeholder" => {
            "\u{30d1}\u{30b9}\u{3067}\u{30d5}\u{30a3}\u{30eb}\u{30bf}\u{30fc}\u{2026}"
        } // パスでフィルター…
        "cache.column.directory" => "\u{30c7}\u{30a3}\u{30ec}\u{30af}\u{30c8}\u{30ea}", // ディレクトリ
        "cache.column.files" => "\u{30d5}\u{30a1}\u{30a4}\u{30eb}\u{6570}", // ファイル数
        "cache.column.size" => "\u{30e1}\u{30c7}\u{30a3}\u{30a2}\u{30b5}\u{30a4}\u{30ba}", // メディアサイズ
        "cache.column.cached_at" => "\u{30ad}\u{30e3}\u{30c3}\u{30b7}\u{30e5}\u{65e5}\u{6642}", // キャッシュ日時
        "cache.footprint" => "\u{30ad}\u{30e3}\u{30c3}\u{30b7}\u{30e5}\u{4f7f}\u{7528}\u{91cf}", // キャッシュ使用量
        "cache.footprint.unavailable" => "\u{53d6}\u{5f97}\u{3067}\u{304d}\u{307e}\u{305b}\u{3093}", // 取得できません
        "cache.load_error" => {
            "\u{30ad}\u{30e3}\u{30c3}\u{30b7}\u{30e5}\u{30da}\u{30fc}\u{30b8}\u{306e}\u{518d}\u{8aad}\u{307f}\u{8fbc}\u{307f}\u{306b}\u{5931}\u{6557}\u{3057}\u{307e}\u{3057}\u{305f}"
        } // キャッシュページの再読み込みに失敗しました
        "cache.load_error.stale" => {
            "\u{30ad}\u{30e3}\u{30c3}\u{30b7}\u{30e5}\u{30da}\u{30fc}\u{30b8}\u{306e}\u{518d}\u{8aad}\u{307f}\u{8fbc}\u{307f}\u{306b}\u{5931}\u{6557}\u{3057}\u{307e}\u{3057}\u{305f}\u{3002}\u{53e4}\u{3044}\u{30c7}\u{30fc}\u{30bf}\u{3092}\u{8868}\u{793a}\u{3057}\u{3066}\u{3044}\u{307e}\u{3059}"
        } // キャッシュページの再読み込みに失敗しました。古いデータを表示しています
        "cache.prune.placeholder" => "\u{76ee}\u{6a19}", // 目標
        "cache.prune.unit_mib" => "MiB",
        "cache.prune.button" => "\u{524a}\u{6e1b}", // 削減
        "cache.prune.done" => "\u{524a}\u{6e1b}\u{3057}\u{307e}\u{3057}\u{305f}", // 削減しました
        "cache.prune.partial" => {
            "\u{524a}\u{6e1b}\u{3057}\u{307e}\u{3057}\u{305f}\u{304c}\u{76ee}\u{6a19}\u{306b}\u{9054}\u{3057}\u{307e}\u{305b}\u{3093}"
        } // 削減しましたが目標に達しません
        "cache.prune.entries" => "\u{4ef6}",        // 件
        "cache.prune.unreclaimable" => "\u{56de}\u{53ce}\u{5bfe}\u{8c61}\u{5916}", // 回収対象外
        "cache.row.caching" => "\u{23f3} \u{30ad}\u{30e3}\u{30c3}\u{30b7}\u{30e5}\u{4e2d}\u{2026}", // ⏳ キャッシュ中…
        "cache.row.stop" => "\u{505c}\u{6b62}", // 停止
        "cache.empty" => {
            "\u{30ad}\u{30e3}\u{30c3}\u{30b7}\u{30e5}\u{3055}\u{308c}\u{305f}\u{30c7}\u{30a3}\u{30ec}\u{30af}\u{30c8}\u{30ea}\u{306f}\u{3042}\u{308a}\u{307e}\u{305b}\u{3093}\u{3002}"
        } // キャッシュされたディレクトリはありません。
        "cache.no_match" => "\u{4e00}\u{81f4}\u{306a}\u{3057}\u{3002}", // 一致なし。
        "cache.summary.directories" => "\u{30c7}\u{30a3}\u{30ec}\u{30af}\u{30c8}\u{30ea}", // ディレクトリ
        "cache.summary.files" => "\u{30d5}\u{30a1}\u{30a4}\u{30eb}", // ファイル
        "cache.summary.total" => "\u{30e1}\u{30c7}\u{30a3}\u{30a2}\u{5408}\u{8a08}", // メディア合計
        "cache.time.just_now" => "\u{305f}\u{3063}\u{305f}\u{4eca}", // たった今
        "cache.time.ago" => "\u{524d}",                              // 前
        "cache.time.minute" => "\u{5206}",                           // 分
        "cache.time.minutes" => "\u{5206}",                          // 分
        "cache.time.hour" => "\u{6642}\u{9593}",                     // 時間
        "cache.time.hours" => "\u{6642}\u{9593}",                    // 時間
        "cache.time.day" => "\u{65e5}",                              // 日
        "cache.time.days" => "\u{65e5}",                             // 日
        "cache.time.month" => "\u{304b}\u{6708}",                    // か月
        "cache.time.months" => "\u{304b}\u{6708}",                   // か月
        "cache.time.year" => "\u{5e74}",                             // 年
        "cache.time.years" => "\u{5e74}",                            // 年

        // Aside tree toggle
        "aside.toggle.open" => "フォルダーツリーを開く", // フォルダーツリーを開く
        "aside.toggle.close" => "フォルダーツリーを閉じる", // フォルダーツリーを閉じる
        // Nav rail tooltips
        "nav.explorer" => "\u{30a8}\u{30af}\u{30b9}\u{30d7}\u{30ed}\u{30fc}\u{30e9}\u{30fc}", // エクスプローラー
        "nav.cache" => "\u{30ad}\u{30e3}\u{30c3}\u{30b7}\u{30e5}", // キャッシュ
        "nav.settings" => "\u{8a2d}\u{5b9a}",                      // 設定

        // Setup wizard
        "setup.download" => "\u{30c0}\u{30a6}\u{30f3}\u{30ed}\u{30fc}\u{30c9}", // ダウンロード
        "setup.skip" => "\u{30b9}\u{30ad}\u{30c3}\u{30d7}",                     // スキップ
        "setup.no_space" => {
            "\u{30c0}\u{30a6}\u{30f3}\u{30ed}\u{30fc}\u{30c9}\u{306b}\u{5341}\u{5206}\u{306a}\u{30c7}\u{30a3}\u{30b9}\u{30af}\u{7a7a}\u{304d}\u{5bb9}\u{91cf}\u{304c}\u{3042}\u{308a}\u{307e}\u{305b}\u{3093}\u{3002}"
        } // ダウンロードに十分なディスク空き容量がありません。
        "setup.item.clip" => {
            "\u{753b}\u{50cf}\u{89e3}\u{6790}AI\u{30e2}\u{30c7}\u{30eb}\u{ff08}CLIP\u{ff09}"
        } // 画像解析AIモデル（CLIP）
        "setup.item.wav2vec2" => {
            "\u{97f3}\u{58f0}\u{89e3}\u{6790}AI\u{30e2}\u{30c7}\u{30eb}\u{ff08}wav2vec2\u{ff09}"
        } // 音声解析AIモデル（wav2vec2）
        "setup.item.ffmpeg" => {
            "\u{52d5}\u{753b}\u{51e6}\u{7406}\u{30bd}\u{30d5}\u{30c8}\u{ff08}ffmpeg\u{ff09}"
        } // 動画処理ソフト（ffmpeg）
        "setup.item.size_unknown" => "\u{ff08}\u{4e0d}\u{660e}\u{ff09}",        // （不明）
        "setup.status.missing" => "\u{672a}\u{53d6}\u{5f97}",                   // 未取得
        "setup.status.checking" => "\u{78ba}\u{8a8d}\u{4e2d}\u{2026}",
        "setup.status.ffmpeg_worker_draining" => {
            "\u{524d}\u{306e}FFmpeg\u{78ba}\u{8a8d}\u{306e}\u{505c}\u{6b62}\u{5f85}\u{3061}\u{2026}"
        }
        "setup.status.external_required" => {
            "\u{5916}\u{90e8}\u{30a4}\u{30f3}\u{30b9}\u{30c8}\u{30fc}\u{30eb}\u{304c}\u{5fc5}\u{8981}"
        }
        "setup.status.downloading" => "\u{30c0}\u{30a6}\u{30f3}\u{30ed}\u{30fc}\u{30c9}\u{4e2d}...", // ダウンロード中...
        "setup.status.ready" => "\u{4f7f}\u{7528}\u{53ef}\u{80fd}", // 使用可能
        "setup.status.error" => "\u{30a8}\u{30e9}\u{30fc}",         // エラー
        "setup.not_ready" => "\u{672a}\u{6e96}\u{5099}\u{ff1a}",    // 未準備：
        "setup.ready" => "\u{6e96}\u{5099}\u{6e08}\u{307f}\u{ff1a}", // 準備済み：
        "setup.download_into" => {
            "AI\u{30e2}\u{30c7}\u{30eb}\u{306e}\u{30c0}\u{30a6}\u{30f3}\u{30ed}\u{30fc}\u{30c9}\u{5148}\u{ff1a}" // AIモデルのダウンロード先：
        }
        "setup.disk_space" => "\u{30c7}\u{30a3}\u{30b9}\u{30af}\u{5bb9}\u{91cf}", // ディスク容量
        "setup.disk_gb_avail" => "GB \u{7a7a}\u{304d}",                           // GB 空き
        "setup.disk_gb_total" => "GB \u{5408}\u{8a08}",                           // GB 合計
        "setup.ffmpeg.external_help" => {
            "\u{304a}\u{4f7f}\u{3044}\u{306e}\u{30d7}\u{30e9}\u{30c3}\u{30c8}\u{30d5}\u{30a9}\u{30fc}\u{30e0}\u{7528}\u{306e}\u{4fe1}\u{983c}\u{3067}\u{304d}\u{308b}\u{914d}\u{5e03}\u{5143}\u{304b}\u{3089}\u{3001}\u{540c}\u{3058}\u{30d0}\u{30fc}\u{30b8}\u{30e7}\u{30f3}\u{306e}ffmpeg\u{3068}ffprobe\u{3092}\u{30a4}\u{30f3}\u{30b9}\u{30c8}\u{30fc}\u{30eb}\u{3057}\u{3001}\u{518d}\u{78ba}\u{8a8d}\u{3057}\u{3066}\u{304f}\u{3060}\u{3055}\u{3044}\u{3002}\u{753b}\u{50cf}\u{306e}\u{307f}\u{306e}\u{5229}\u{7528}\u{306f}\u{305d}\u{306e}\u{307e}\u{307e}\u{7d9a}\u{884c}\u{3067}\u{304d}\u{307e}\u{3059}\u{3002}"
        }
        "setup.ffmpeg.recheck" => "\u{518d}\u{78ba}\u{8a8d}",
        "setup.ffmpeg.select" => {
            "ffmpeg \u{30d5}\u{30a9}\u{30eb}\u{30c0}\u{30fc}\u{3092}\u{9078}\u{629e}"
        }

        // Focus dialog
        "focus.strategy" => {
            "\u{30ad}\u{30e3}\u{30c3}\u{30b7}\u{30e5}\u{691c}\u{7d22}\u{7bc4}\u{56f2}"
        } // キャッシュ検索範囲
        "focus.close" => "\u{9589}\u{3058}\u{308b}", // 閉じる

        // Similar-pairs dialog
        "pairs.no_valid" => {
            "\u{985e}\u{4f3c}\u{30da}\u{30a2}\u{306f}\u{3042}\u{308a}\u{307e}\u{305b}\u{3093}\u{3002}"
        } // 類似ペアはありません。

        // Header
        "header.folder" => "\u{30d5}\u{30a9}\u{30eb}\u{30c0}\u{30fc}", // フォルダー

        // Gallery
        "gallery.empty" => {
            "\u{8868}\u{793a}\u{3059}\u{308b}\u{30d5}\u{30a1}\u{30a4}\u{30eb}\u{304c}\u{3042}\u{308a}\u{307e}\u{305b}\u{3093}\u{3002}"
        } // 表示するファイルがありません。

        // Gallery filter
        "gallery.filter.placeholder" => "ファイル名でフィルター…",
        "gallery.filter.clear" => "✕",
        "gallery.filter.count_of" => "件中",

        // Theme selector (RFC 011)
        "settings.general.theme" => "テーマ",
        "settings.general.theme.light" => "ライト",
        "settings.general.theme.dark" => "ダーク",
        "settings.general.theme.hc_light" => "ハイコントラスト（明）",
        "settings.general.theme.hc_dark" => "ハイコントラスト（暗）",
        "settings.general.theme.hc_note" => {
            "ハイコントラストは標準ウィジェットの基本色にも反映されます。arama 独自のコントロールには完全なハイコントラストパレットが適用されます。"
        }

        // Startup
        "startup.local_setup_error.title" => "起動時のセットアップ確認に失敗しました",
        "startup.local_setup_error.body" => {
            "ローカルセットアップディレクトリを準備できませんでした"
        }
        "startup.root_dir_unavailable.title" => "起動時のフォルダーを開けません",
        "startup.root_dir_unavailable.body" => {
            "保存されたフォルダーを開けません。別のフォルダーを選択するとインデックス作成を開始できます"
        }
        "startup.root_scan_warning.title" => "フォルダーのスキャンが一部失敗しました",
        "startup.root_scan_warning.body" => "起動時に一部のフォルダーを読み取れませんでした",

        _ => return None,
    })
}
