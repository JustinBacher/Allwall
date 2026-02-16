#[derive(thiserror::Error, Debug)]
pub enum SourceError {
    #[error("Image load failed: {0}")]
    ImageLoad(#[from] image::ImageError),

    #[error("Failed to acquire surface texture: {0}")]
    SurfaceAcquire(String),

    #[error("Failed to create texture {width}x{height}")]
    TextureCreate { width: u32, height: u32 },

    #[error("Failed to create render pipeline for {0}")]
    PipelineCreate(&'static str),

    #[error("Failed to create bind group for {0}")]
    BindGroupCreate(&'static str),

    #[error("Failed to create buffer: {0}")]
    BufferCreate(String),

    #[error("Simulation grid overflow - dimensions too large")]
    GridOverflow,

    #[error("Video decode error: {0}")]
    VideoDecode(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("No image directory configured")]
    NoImageDirectory,

    #[error("No images available")]
    NoImagesAvailable,

    #[error("No previous image in history")]
    NoPreviousImage,
}
