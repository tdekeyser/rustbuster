use std::fmt::{Display, Formatter};

use derive_more::From;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    #[from]
    Custom(String),

    FuzzKeywordNotFound,

    #[from]
    Io(std::io::Error),

    #[from]
    Http(reqwest::Error),

    #[from]
    HttpHeaderNameInvalid(reqwest::header::InvalidHeaderName),

    #[from]
    HttpHeaderValueInvalid(reqwest::header::InvalidHeaderValue),

    #[from]
    BruteForceError(tokio::task::JoinError)
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::Custom(value.to_string())
    }
}

// error boilerplate
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}
