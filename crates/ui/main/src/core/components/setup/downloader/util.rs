use std::path::PathBuf;

use arama_ai::model::model_container::{ModelContainer, SourceUrl};
use arama_env::validate_dir;
use iced::futures::{SinkExt, StreamExt, channel::mpsc::Sender};
use tokio::fs::{self, File};
use tokio::io::{AsyncWriteExt, BufWriter};

use super::{config::DownloaderConfig, state::DownloadProgress};

/// 一般ファイル: ダウンロードを行い、進捗を Stream として返す
pub fn general_download_stream(
    url: String,
    download_dest_path: PathBuf,
    downloader_config: DownloaderConfig,
) -> impl StreamExt<Item = DownloadProgress> {
    iced::stream::channel(
        100,
        move |mut output: Sender<DownloadProgress>| async move {
            // 1. 上書き防止チェック (Downloader struct 初期化時と複合チェック)
            if download_dest_path.exists() {
                let _ = output
                    .send(DownloadProgress::Errored(
                        "ファイルが既に存在します".to_string(),
                    ))
                    .await;
                return;
            }

            // 2. HTTPリクエスト
            let response = match reqwest::get(url).await {
                Ok(res) => res,
                Err(e) => {
                    let _ = output
                        .send(DownloadProgress::Errored(format!("通信エラー: {}", e)))
                        .await;
                    return;
                }
            };

            if !response.status().is_success() {
                let _ = output
                    .send(DownloadProgress::Errored(format!(
                        "HTTPエラー: {}",
                        response.status()
                    )))
                    .await;
                return;
            }

            let total_size = response.content_length().unwrap_or(0) as f32;
            let mut downloaded = 0.0;

            // 3. 一時ファイルの作成とBufWriterの準備
            let parent_dir = download_dest_path.parent();
            if parent_dir.is_none() || !validate_dir(&parent_dir.unwrap()).is_ok() {
                let _ = output
                    .send(DownloadProgress::Errored(format!(
                        "親フォルダ確保失敗: {}",
                        download_dest_path.to_string_lossy()
                    )))
                    .await;
                return;
            }
            let part_name = format!("{}.part", download_dest_path.to_string_lossy().to_string());
            let file = match File::create(&part_name).await {
                Ok(f) => f,
                Err(e) => {
                    let _ = output
                        .send(DownloadProgress::Errored(format!(
                            "ファイル作成失敗: {}",
                            e
                        )))
                        .await;
                    return;
                }
            };
            let mut writer = BufWriter::new(file);
            let mut stream = response.bytes_stream();

            // 4. データ受信と書き込みループ
            while let Some(item) = stream.next().await {
                match item {
                    Ok(chunk) => {
                        if let Err(e) = writer.write_all(&chunk).await {
                            let _ = output
                                .send(DownloadProgress::Errored(format!("書き込み失敗: {}", e)))
                                .await;
                            let _ = fs::remove_file(&part_name).await;
                            return;
                        }

                        downloaded += chunk.len() as f32;
                        let percentage = if total_size > 0.0 {
                            (downloaded / total_size) * 100.0
                        } else {
                            0.0
                        };
                        let _ = output.send(DownloadProgress::Downloading(percentage)).await;
                    }
                    Err(e) => {
                        let _ = output
                            .send(DownloadProgress::Errored(format!("受信中断: {}", e)))
                            .await;
                        let _ = fs::remove_file(&part_name).await;
                        return;
                    }
                }
            }

            // 5. 書き込み確定処理
            if let Err(e) = writer.flush().await {
                let _ = output
                    .send(DownloadProgress::Errored(format!("保存失敗: {}", e)))
                    .await;
                let _ = fs::remove_file(&part_name).await;
                return;
            }

            // ファイルロック解放のため明示的にdrop
            drop(writer);

            // 6. 完了後のリネーム
            if let Err(e) = fs::rename(&part_name, &download_dest_path).await {
                let _ = output
                    .send(DownloadProgress::Errored(format!("リネーム失敗: {}", e)))
                    .await;
            }

            // すべての処理（モデル＋任意の設定ファイル）が完了したら通知
            let _ = output
                .send(DownloadProgress::Finished(downloader_config))
                .await;
        },
    )
}

/// AI model: ダウンロードを行い、進捗を Stream として返す
pub fn ai_model_download_stream(
    model_container: ModelContainer,
) -> impl StreamExt<Item = DownloadProgress> {
    iced::stream::channel(
        100,
        move |mut output: Sender<DownloadProgress>| async move {
            let safetensors_path = model_container
                .safetensors_path()
                .expect("failed to get safetensors path");

            // 1. 上書き防止チェック (Downloader struct 初期化時と複合チェック)
            if safetensors_path.exists() {
                let _ = output
                    .send(DownloadProgress::Errored(
                        "ファイルが既に存在します".to_string(),
                    ))
                    .await;
                return;
            }

            // 2. HTTPリクエスト
            let (model_url, path_to_save) = match &model_container.source_url {
                SourceUrl::ModelSafetensors(x) | SourceUrl::ModelSafetensorsConfigJson((x, _)) => (
                    x,
                    model_container
                        .safetensors_path()
                        .expect("failed to get pytorch path"),
                ),
                SourceUrl::PyTorch(x) => (
                    x,
                    model_container
                        .pytorch_path()
                        .expect("failed to get pytorch path"),
                ),
            };

            let response = match reqwest::get(model_url).await {
                Ok(res) => res,
                Err(e) => {
                    let _ = output
                        .send(DownloadProgress::Errored(format!("通信エラー: {}", e)))
                        .await;
                    return;
                }
            };

            if !response.status().is_success() {
                let _ = output
                    .send(DownloadProgress::Errored(format!(
                        "HTTPエラー: {}",
                        response.status()
                    )))
                    .await;
                return;
            }

            let total_size = response.content_length().unwrap_or(0) as f32;
            let mut downloaded = 0.0;

            // 3. 一時ファイルの作成とBufWriterの準備
            let parent_dir = path_to_save.parent();
            if parent_dir.is_none() || !validate_dir(&parent_dir.unwrap()).is_ok() {
                let _ = output
                    .send(DownloadProgress::Errored(format!(
                        "親フォルダ確保失敗: {}",
                        path_to_save.to_string_lossy()
                    )))
                    .await;
                return;
            }
            let part_name = format!("{}.part", path_to_save.to_string_lossy().to_string());
            let file = match File::create(&part_name).await {
                Ok(f) => f,
                Err(e) => {
                    let _ = output
                        .send(DownloadProgress::Errored(format!(
                            "ファイル作成失敗: {}",
                            e
                        )))
                        .await;
                    return;
                }
            };
            let mut writer = BufWriter::new(file);
            let mut stream = response.bytes_stream();

            // 4. データ受信と書き込みループ
            while let Some(item) = stream.next().await {
                match item {
                    Ok(chunk) => {
                        if let Err(e) = writer.write_all(&chunk).await {
                            let _ = output
                                .send(DownloadProgress::Errored(format!("書き込み失敗: {}", e)))
                                .await;
                            let _ = fs::remove_file(&part_name).await;
                            return;
                        }

                        downloaded += chunk.len() as f32;
                        let percentage = if total_size > 0.0 {
                            (downloaded / total_size) * 100.0
                        } else {
                            0.0
                        };
                        let _ = output.send(DownloadProgress::Downloading(percentage)).await;
                    }
                    Err(e) => {
                        let _ = output
                            .send(DownloadProgress::Errored(format!("受信中断: {}", e)))
                            .await;
                        let _ = fs::remove_file(&part_name).await;
                        return;
                    }
                }
            }

            // 5. 書き込み確定処理
            if let Err(e) = writer.flush().await {
                let _ = output
                    .send(DownloadProgress::Errored(format!("保存失敗: {}", e)))
                    .await;
                let _ = fs::remove_file(&part_name).await;
                return;
            }

            // ファイルロック解放のため明示的にdrop
            drop(writer);

            // 6. 完了後のリネーム
            if let Err(e) = fs::rename(&part_name, &path_to_save).await {
                let _ = output
                    .send(DownloadProgress::Errored(format!("リネーム失敗: {}", e)))
                    .await;
            }

            let _ = model_container
                .ensure_safetensors()
                .expect("failed to ensure safetensors");

            // config_url
            match &model_container.source_url {
                SourceUrl::ModelSafetensorsConfigJson((_, config_url)) => {
                    // 軽量ファイルなので一括でメモリに取得
                    match reqwest::get(config_url).await {
                        Ok(res) => {
                            if !res.status().is_success() {
                                let _ = output
                                    .send(DownloadProgress::Errored(format!(
                                        "設定ファイルのHTTPエラー: {}",
                                        res.status()
                                    )))
                                    .await;
                                return;
                            }

                            match res.bytes().await {
                                Ok(bytes) => {
                                    // .part を経由せず、直接指定パスに一括書き込み
                                    let url = reqwest::Url::parse(&config_url).unwrap();

                                    // path_segments() は '?' 以降（クエリパラメーター）を自動的に除外します
                                    let filename = url
                                        .path_segments() // パスをセグメント（["user", "repo", ..., "model.safetensors"]）に分解
                                        .and_then(|s| s.last()) // 最後の要素を取得
                                        .filter(|s| !s.is_empty()) // 末尾がスラッシュで終わるケース（.../path/?q=1）を排除
                                        .unwrap_or("model.bin");

                                    let path = parent_dir.unwrap().join(&filename);

                                    if let Err(e) = fs::write(&path, bytes).await {
                                        let _ = output
                                            .send(DownloadProgress::Errored(format!(
                                                "設定の保存失敗: {}",
                                                e
                                            )))
                                            .await;
                                        return;
                                    }
                                }
                                Err(e) => {
                                    let _ = output
                                        .send(DownloadProgress::Errored(format!(
                                            "設定の読み込みエラー: {}",
                                            e
                                        )))
                                        .await;
                                    return;
                                }
                            }
                        }
                        Err(e) => {
                            let _ = output
                                .send(DownloadProgress::Errored(format!(
                                    "設定ファイルの通信エラー: {}",
                                    e
                                )))
                                .await;
                            return;
                        }
                    }
                }
                _ => (),
            }

            // すべての処理（モデル＋任意の設定ファイル）が完了したら通知
            let _ = output
                .send(DownloadProgress::Finished(DownloaderConfig::AiModel(
                    model_container,
                )))
                .await;
        },
    )
}
