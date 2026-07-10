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

use arama_embedding::{
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
            // Build and return the worker stream here.
            stream::channel(100, |mut output: Sender<Message>| async move {
                let (sender, mut receiver) = mpsc::channel::<Input>(100);

                // Notify the UI that the worker is ready.
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

    // // 1. Wrap the Rayon work in spawn_blocking.
    // // This keeps the iced (tokio) executor from being blocked.
    // let _ = tokio::task::spawn_blocking(move || {
    //     // 2. Run the iterator in parallel with Rayon.
    //     match dir_node_image_similarity(&dir_node, &calculator) {
    //         Ok(x) => {
    //             // 3. Return the results either batched or item by item.
    //             for item in x {
    //                 // Async channel sends may require block_on, but simple
    //                 // sends can use try_send or per-item tasks.
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
//         // 2. Run the iterator in parallel with Rayon.
//         .into_par_iter()
//         .map(|target| {
//             // CPU-intensive work.
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
