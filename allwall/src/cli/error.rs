#[derive(thiserror::Error, Debug)]
pub enum CliError {
    #[error("Daemon is already running - only one instance allowed")]
    DaemonRunning,

    #[error("Daemon is not running - start with 'allwall run'")]
    DaemonNotRunning,

    #[error("Failed to connect to daemon socket at {path}: {source}")]
    SocketConnect {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to create socket address")]
    SocketAddrCreate,

    #[error("IPC request serialization failed: {0}")]
    RequestSerialize(String),

    #[error("IPC response deserialization failed: {0}")]
    ResponseDeserialize(String),

    #[error("IPC error: {0}")]
    Ipc(String),

    #[error("Failed to generate shell completions for {shell}")]
    Completions { shell: String },

    #[error("--path is required for media source. Use --path <PATH>")]
    MediaPathRequired,
}
