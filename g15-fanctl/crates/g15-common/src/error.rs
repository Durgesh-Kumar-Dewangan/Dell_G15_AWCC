use thiserror::Error;

pub type Result<T> = std::result::Result<T, G15Error>;

#[derive(Debug, Error)]
pub enum G15Error {
    #[error("hardware interface not found: {0}")]
    NotFound(String),

    #[error("hardware interface present but read-only, cannot perform: {0}")]
    ReadOnly(String),

    #[error("unsupported on this firmware/model: {0}")]
    Unsupported(String),

    #[error("value {value} out of safe range [{min}, {max}] for {what}")]
    OutOfRange {
        what: String,
        value: i64,
        min: i64,
        max: i64,
    },

    #[error("I/O error accessing {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse value from {path}: {value:?}")]
    Parse { path: String, value: String },

    #[error("permission denied accessing {0} (daemon must run with appropriate privileges)")]
    PermissionDenied(String),

    #[error("other: {0}")]
    Other(#[from] anyhow::Error),
}

impl G15Error {
    pub fn io(path: impl Into<String>, source: std::io::Error) -> Self {
        let path = path.into();
        if source.kind() == std::io::ErrorKind::PermissionDenied {
            G15Error::PermissionDenied(path)
        } else {
            G15Error::Io { path, source }
        }
    }
}
