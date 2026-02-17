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

use crate::engine::{
    store::file::file_embedding::FileEmbedding,
    // pipeline::clip::inference::{self, calculator::Calculator, clip},
};

use super::{Gallery, message::Message};

#[derive(Debug, Clone)]
pub enum Input {
    // ImageSimilarity((PathBuf, DirNode)),
    ImageSimilarity(DirNode),
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
                        // Input::ImageSimilarity((source, dir_node)) => {
                        Input::ImageSimilarity(dir_node) => {
                            let output = output.clone();
                            // let _ = image_similarity(output, source, dir_node);
                            let _ = image_similarity(output, dir_node);
                        }
                    }
                }
            })
        })
    }
}

fn image_similarity(
    mut output: Sender<Message>,
    // source: PathBuf,
    dir_node: DirNode,
) -> anyhow::Result<()> {
    // // let calculator = inference::calculator(source.as_path())?;
    // let calculator = inference::calculator()?;

    // // 1. Rayonの処理を spawn_blocking で包む
    // // これにより iced (tokio) の executor をブロックせずに済む
    // let _ = tokio::task::spawn_blocking(move || {
    //     // 2. Rayon で並列計算 (イテレータを parallel に)
    //     match dir_node_image_similarity(&dir_node, &calculator) {
    //         Ok(x) => {
    //             // 3. まとめて、あるいは個別に結果を返す
    //             for item in x {
    //                 // 非同期チャネルへ送るために block_on 等が必要になる場合があるが
    //                 // 単純な送信なら try_send や個別のタスク化で対応
    //                 let _ = output.try_send(Message::SubscriptionWorkerFinished(item));
    //             }
    //         }
    //         Err(_) => {
    //             let _ = output.try_send(Message::SubscriptionWorkerFailed);
    //         }
    //     }
    // });

    Ok(())
}

// fn dir_node_image_similarity(
//     dir_node: &DirNode,
//     calculator: &Calculator,
// ) -> anyhow::Result<Vec<FileEmbedding>> {
//     let mut ret: Vec<FileEmbedding> = vec![];

//     ret = dir_node
//         .files
//         .clone()
//         // 2. Rayon で並列計算 (イテレータを parallel に)
//         .into_par_iter()
//         .map(|target| {
//             // CPUを酷使する重い計算...
//             match clip(&target, &calculator) {
//                 Ok(x) => FileEmbedding {
//                     path: x.path,
//                     embedding: x.embedding,
//                 },
//                 Err(err) => {
//                     // todo: error handling
//                     eprint!("{}", err);
//                     FileEmbedding {
//                         path: PathBuf::new(),
//                         embedding: vec![],
//                     }
//                 }
//             }
//         })
//         .collect();

//     ret.extend(
//         dir_node
//             .sub_dirs
//             .clone()
//             .into_par_iter()
//             .map(
//                 |dir_node| match dir_node_image_similarity(&dir_node, calculator) {
//                     Ok(x) => x,
//                     Err(_) => vec![],
//                 },
//             )
//             .collect::<Vec<Vec<FileEmbedding>>>()
//             .into_iter()
//             .flatten()
//             .collect::<Vec<FileEmbedding>>(),
//     );

//     Ok(ret)
// }
