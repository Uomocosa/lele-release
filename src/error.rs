use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("git failed in {repo}: {command}\n{stderr}")]
    Git {
        repo: String,
        command: String,
        stderr: String,
    },

    #[error("i/o error: {message}")]
    Io { message: String },

    #[error("cannot parse {path}: {message}")]
    Toml { path: String, message: String },

    #[error("{0}")]
    Refused(String),
}
