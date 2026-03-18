use arama_ai::model::model_container::ModelContainer;

#[derive(Debug, Clone)]
pub enum DownloaderConfig {
    AiModel(ModelContainer),
    Ffmepg,
}
