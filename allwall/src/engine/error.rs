#[derive(thiserror::Error, Debug)]
pub enum EngineError {
    #[error("Failed to connect to Wayland: {0}")]
    WaylandConnect(String),

    #[error("Wayland registry initialization failed: {0}")]
    WaylandRegistry(String),

    #[error("Compositor not available - ensure a Wayland compositor is running")]
    NoCompositor,

    #[error("Layer shell protocol not available")]
    NoLayerShell,

    #[error("Event loop creation failed: {0}")]
    EventLoopCreate(String),

    #[error("Signal handler registration failed")]
    SignalHandler,

    #[error("Media source requires --path argument")]
    MediaPathRequired,

    #[error("No image directory configured")]
    NoImageDirectory,

    #[error("Failed to read directory {path}: {source}")]
    DirectoryRead {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to insert Wayland source into event loop: {0}")]
    WaylandSourceInsert(String),

    #[error("Render error: {0}")]
    Render(String),

    #[error("No scenes configured")]
    NoScenes,
}

#[derive(thiserror::Error, Debug)]
pub enum ContextError {
    #[error("Invalid Wayland surface pointer - surface may be destroyed")]
    InvalidSurfacePointer,

    #[error("Invalid Wayland display pointer")]
    InvalidDisplayPointer,

    #[error("GPU surface creation failed: {0}")]
    SurfaceCreate(String),

    #[error("No suitable GPU adapter found")]
    NoAdapter,

    #[error("GPU device creation failed: {0}")]
    DeviceCreate(String),

    #[error("Surface configuration failed during resize: {0}")]
    SurfaceConfig(String),

    #[error("No SRGB surface format available")]
    NoSrgbFormat,

    #[error(transparent)]
    Wgpu(#[from] wgpu::Error),
}
