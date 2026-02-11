use iced::{
    Subscription,
    futures::{
        SinkExt, StreamExt,
        channel::mpsc::{self, Sender},
    },
    stream,
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use swdir::DirNode;

use std::path::PathBuf;

use super::{Gallery, message::Message};
use crate::app::utils::gallery::image_similarity::{
    Calculator, ImageSimilarity, calculate, calculator_prepare,
};

#[derive(Debug, Clone)]
pub enum Input {
    ImageSimilarity((PathBuf, DirNode)),
}

impl Gallery {
    pub fn subscription(&self) -> Subscription<Message> {
        if !self.processing {
            return Subscription::none();
        }

        Subscription::run(|| {
            // ここでストリームを生成して返す
            stream::channel(100, |mut output: Sender<Message>| async move {
                let (sender, mut receiver) = mpsc::channel::<Input>(100);

                // 準備完了通知
                let _ = output.send(Message::SubscriptionWorkerReady(sender)).await;

                while let Some(input) = receiver.next().await {
                    match input {
                        Input::ImageSimilarity((source, dir_node)) => {
                            let output = output.clone();
                            let _ = image_similarity(output, source, dir_node);
                        }
                    }
                }
            })
        })
    }
}

fn image_similarity(
    mut output: Sender<Message>,
    source: PathBuf,
    dir_node: DirNode,
) -> anyhow::Result<()> {
    let calculator = calculator_prepare(source.as_path())?;

    // 1. Rayonの処理を spawn_blocking で包む
    // これにより iced (tokio) の executor をブロックせずに済む
    let _ = tokio::task::spawn_blocking(move || {
        // 2. Rayon で並列計算 (イテレータを parallel に)
        match dir_node_image_similarity(&dir_node, &calculator) {
            Ok(x) => {
                // 3. まとめて、あるいは個別に結果を返す
                for item in x {
                    // 非同期チャネルへ送るために block_on 等が必要になる場合があるが
                    // 単純な送信なら try_send や個別のタスク化で対応
                    let _ = output.try_send(Message::SubscriptionWorkerFinished(item));
                }
            }
            Err(_) => {
                let _ = output.try_send(Message::SubscriptionWorkerFailed);
            }
        }
    });

    Ok(())
}

fn dir_node_image_similarity(
    dir_node: &DirNode,
    calculator: &Calculator,
) -> anyhow::Result<Vec<ImageSimilarity>> {
    let mut ret: Vec<ImageSimilarity> = vec![];

    ret = dir_node
        .files
        .clone()
        // 2. Rayon で並列計算 (イテレータを parallel に)
        .into_par_iter()
        .map(|target| {
            // CPUを酷使する重い計算...
            match calculate(&target, &calculator) {
                Ok(x) => ImageSimilarity {
                    path: x.path,
                    score: x.score,
                },
                Err(err) => {
                    // todo: error handling
                    eprint!("{}", err);
                    ImageSimilarity {
                        path: PathBuf::new(),
                        score: 0.0,
                    }
                }
            }
        })
        .collect();

    ret.extend(
        dir_node
            .sub_dirs
            .clone()
            .into_par_iter()
            .map(
                |dir_node| match dir_node_image_similarity(&dir_node, calculator) {
                    Ok(x) => x,
                    Err(_) => vec![],
                },
            )
            .collect::<Vec<Vec<ImageSimilarity>>>()
            .into_iter()
            .flatten()
            .collect::<Vec<ImageSimilarity>>(),
    );

    Ok(ret)
}
