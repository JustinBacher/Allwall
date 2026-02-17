#[derive(thiserror::Error, Debug)]
pub enum VideoError {
    #[error("Failed to parse GStreamer pipeline: {0}")]
    PipelineParse(String),

    #[error("Pipeline downcast failed - expected Pipeline object")]
    PipelineDowncast,

    #[error("AppSink element '{0}' not found in pipeline")]
    SinkNotFound(&'static str),

    #[error("Failed to cast element to AppSink")]
    SinkCast,

    #[error("Failed to start pipeline: {0}")]
    PipelineStart(String),

    #[error("Video file not found: {0}")]
    FileNotFound(std::path::PathBuf),

    #[error("Failed to read video file: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("Failed to pull sample from appsink: {0}")]
    SamplePull(String),

    #[error("Failed to get buffer from sample")]
    BufferNotFound,

    #[error("Failed to map buffer memory: {0}")]
    BufferMap(String),

    #[error("No video frames available")]
    NoFrames,

    #[error("Texture creation failed: {0}")]
    TextureCreation(String),

    #[error("Generic error: {0}")]
    Generic(String),
}
