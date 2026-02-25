// use arama_ai::{
//     config::video_similarity_config::VideoSimilarityConfig, model::model_manager::ModelManager,
//     pipeline_manager,
// };

mod core;

pub fn start() -> iced::Result {
    // let c = VideoSimilarityConfig::default();
    // let d = ModelManager::device();
    // let x = pipeline_manager::video_similarity_pipeline::VideoSimilarityPipeline::new(c, d)
    //     .expect("failed to start");
    // let r = x.compare(
    //     std::path::Path::new("sample1.mp4"),
    //     std::path::Path::new("sample2.mp4"),
    // );
    // println!("------------\n{:?}\n------------", r);

    core::App::start()
}
