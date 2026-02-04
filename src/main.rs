use iced::application::IntoBoot;
use iced::widget::{Image, Responsive, button, column, container, image, row, scrollable, text};
use iced::{Element, Length, Size, Task};
use std::any::Any;
use std::path::PathBuf;

pub fn main() -> iced::Result {
    iced::application(Gallery::new, Gallery::update, Gallery::view)
        // .theme(|_| Theme::Dark)
        .run()
}

// アプリケーションの状態
struct Gallery {
    image_paths: Vec<PathBuf>,
    thumbnail_size: u32,
    spacing: u32,
}

// アプリケーションのメッセージ（イベント）
#[derive(Debug, Clone)]
enum Message {
    ImagesLoaded(Vec<PathBuf>),
    ScaleUp,
    ScaleDown,
    Quit,
}

impl Default for Gallery {
    fn default() -> Self {
        Self {
            image_paths: Vec::new(),
            thumbnail_size: 160, // サムネイルの正方形サイズ
            spacing: 10,         // 画像間の隙間
        }
    }
}

impl Gallery {
    // アプリケーション初期化時に画像を読み込むTaskを発行
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ImagesLoaded(paths) => {
                self.image_paths = paths;
                Task::none()
            }
            Message::ScaleUp => {
                if self.thumbnail_size <= 600 {
                    self.thumbnail_size += 20;
                }
                Task::none()
            }
            Message::ScaleDown => {
                if 40 <= self.thumbnail_size {
                    self.thumbnail_size -= 20;
                }
                Task::none()
            }
            Message::Quit => iced::exit(),
        }
    }

    // 初期化ロジック（非同期で画像をスキャン）
    fn new() -> (Self, Task<Message>) {
        (
            Self::default(),
            Task::perform(load_images("."), Message::ImagesLoaded), // カレントディレクトリをスキャン
        )
    }

    // ビュー（UI描画）
    fn view(&self) -> Element<'_, Message> {
        let menus = row![
            button("+").on_press(Message::ScaleUp),
            button("-").on_press(Message::ScaleDown),
            button("x").on_press(Message::Quit),
        ];

        let content = if self.image_paths.is_empty() {
            container(text("No images found in this folder."))
        } else {
            // Responsiveウィジェットを使って、現在のウィンドウ幅(size)を取得する
            container(Responsive::new(move |size| self.view_grid(size)))
        };

        let container = container(content)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

        // スクロール可能にする
        let scrollable = scrollable(container);

        column![menus, scrollable].into()
    }

    // グリッドレイアウトの計算ロジック
    fn view_grid(&self, size: Size) -> Element<'_, Message> {
        let total_width = size.width;
        let item_width = self.thumbnail_size as f32 + self.spacing as f32;

        // 1行に収まるカラム数を計算 (ゼロ除算回避のためmax(1)を使用)
        let columns = (total_width / item_width).floor() as usize;
        let columns = columns.max(1);

        // 画像パスのリストを、カラム数ごとに分割（チャンク化）して行を作成
        let rows: Vec<Element<Message>> = self
            .image_paths
            .chunks(columns)
            .map(|chunk| {
                let images: Vec<Element<Message>> = chunk
                    .iter()
                    .map(|path| {
                        // 画像ウィジェットの作成
                        // ContentFit::Coverで正方形にトリミング表示
                        image(path)
                            .width(self.thumbnail_size)
                            .height(self.thumbnail_size)
                            .content_fit(iced::ContentFit::Cover)
                            .into()
                    })
                    .collect();

                // 画像を横に並べる
                row(images).spacing(self.spacing).into()
            })
            .collect();

        // 行を縦に並べる
        column(rows).spacing(self.spacing).into()
    }
}

// フォルダ内の画像を非同期で検索するヘルパー関数
async fn load_images(dir: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_file() {
                // 拡張子で画像のみをフィルタリング
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    match ext.to_lowercase().as_str() {
                        "png" | "jpg" | "jpeg" | "gif" | "bmp" => paths.push(path),
                        _ => {}
                    }
                }
            }
        }
    }
    paths.sort(); // 名前順にソート
    paths
}

// `application` ビルダーを使用する場合の初期化フック
impl Gallery {
    // 0.12以降の新しいApplication trait構造に合わせてTaskを起動
    // 注: `iced::application` ショートカットを使う場合、
    // ここで初期化コマンドを渡す構成にする必要があります。
    // 今回は簡易化のため、main関数内のロジックで完結させますが、
    // 正式なTrait実装スタイルに合わせる場合は `impl Application for Gallery` を使用します。
}
