pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    LogError(flexi_logger::FlexiLoggerError),
    IoError(std::io::Error),
    JsonError(serde_json::error::Error),
    HttpError(reqwest::Error),
    InvalidUrl(url::ParseError),
    MissingProductCategory(Vec<String>),
    InvalidProductCategory(String, Vec<String>),
    MissingTargetOS(Vec<String>),
    InvalidTargetOS(String, Vec<String>),
    MissingRelease(Vec<String>),
    InvalidRelease(String, Vec<String>),
    L2RepoReleaseMissingUrl(String),
    InvalidSection(String),
    InvalidGroup(String),
    InvalidComponent(String),
}

impl From<flexi_logger::FlexiLoggerError> for Error {
    fn from(err: flexi_logger::FlexiLoggerError) -> Self {
        Error::LogError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(err: serde_json::error::Error) -> Self {
        Error::JsonError(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::HttpError(err)
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self {
        Error::InvalidUrl(err)
    }
}
