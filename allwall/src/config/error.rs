#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file {path}: {source}")]
    Read {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse config file {path}: {source}")]
    Parse {
        path: std::path::PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("Failed to determine XDG config directory: {0}")]
    XdgBaseDir(String),

    #[error("Monitor conflict: monitors {monitors:?} are claimed by multiple scenes")]
    MonitorOverlap { monitors: Vec<String> },

    #[error("Monitor '{name}' claimed by scene conflicts with 'any' selector")]
    MonitorAnyConflict { name: String },

    #[error("Invalid GPU selection '{value}': {reason}")]
    InvalidGpu { value: String, reason: String },
}
