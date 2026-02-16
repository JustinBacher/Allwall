#[derive(thiserror::Error, Debug)]
pub enum TransitionError {
    #[error(
        "Invalid transition type: '{0}' - expected one of: fade, circle-top-left, circle-top-right, circle-bottom-left, circle-bottom-right, circle-center, circle-random"
    )]
    InvalidType(String),

    #[error("Transition duration must be greater than zero")]
    ZeroDuration,

    #[error("Failed to acquire surface texture during transition: {0}")]
    SurfaceAcquire(String),

    #[error("Failed to create shader module for {transition}")]
    ShaderCreate { transition: &'static str },

    #[error("Failed to create render pipeline for {transition}")]
    PipelineCreate { transition: &'static str },

    #[error("Failed to create bind group for {transition}")]
    BindGroupCreate { transition: &'static str },
}
