pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Utf8ParseError(std::str::Utf8Error),
    LogError(flexi_logger::FlexiLoggerError),
    IoError(std::io::Error),
    JsonError(serde_json::error::Error),
    HttpError(reqwest::Error),
    HttpStatusError(http::status::StatusCode),
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
    UnsupportedChecksumType(String),
    FileNotExist(String),
    FileDigestInvalid {
        file: String,
        cktype: String,
        expected: String,
        actual: String,
    },
}

impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Self {
        Error::Utf8ParseError(err)
    }
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

impl From<http::status::StatusCode> for Error {
    fn from(err: http::status::StatusCode) -> Self {
        Error::HttpStatusError(err)
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self {
        Error::InvalidUrl(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Utf8ParseError(e) => write!(f, "ERROR: {}", e),
            Error::LogError(e) => write!(f, "ERROR: {}", e),
            Error::IoError(e) => write!(f, "ERROR: {}", e),
            Error::JsonError(e) => write!(f, "ERROR: {}", e),
            Error::HttpError(e) => write!(f, "ERROR: {}", e),
            Error::HttpStatusError(e) => write!(f, "ERROR: {}", e),
            Error::InvalidUrl(e) => write!(f, "ERROR: {}", e),
            Error::MissingProductCategory(cats) => {
                write!(
                    f,
                    "ERROR: Required option --product-category is missing. Legal values:"
                )?;
                for cat in cats {
                    write!(f, "\t{}", cat)?;
                }
                write!(f, "")
            }
            Error::InvalidProductCategory(e, cats) => {
                write!(
                    f,
                    "ERROR: Invalid value {} for option --product-category. Legal values:",
                    e
                )?;
                for cat in cats {
                    write!(f, "\t{}", cat)?;
                }
                write!(f, "")
            }
            Error::MissingTargetOS(tgts) => {
                write!(
                    f,
                    "ERROR: Required option --target-os is missing. Legal values:"
                )?;
                for tgt in tgts {
                    write!(f, "\t{}", tgt)?;
                }
                write!(f, "")
            }
            Error::InvalidTargetOS(e, tgts) => {
                write!(
                    f,
                    "ERROR: Invalid value {} for option --target-os. Legal values:",
                    e
                )?;
                for tgt in tgts {
                    write!(f, "\t{}", tgt)?;
                }
                write!(f, "")
            }
            Error::MissingRelease(rels) => {
                write!(
                    f,
                    "ERROR: Required option --release is missing. Legal values:"
                )?;
                for rel in rels {
                    write!(f, "\t{}", rel)?;
                }
                write!(f, "")
            }
            Error::InvalidRelease(e, rels) => {
                write!(
                    f,
                    "ERROR: Invalid value {} for option --release. Legal values:",
                    e
                )?;
                for rel in rels {
                    write!(f, "\t{}", rel)?;
                }
                write!(f, "")
            }
            Error::L2RepoReleaseMissingUrl(url) => write!(
                f,
                "ERROR: The L2 repo doesn't specify a URL for the requested release {}.",
                url
            ),
            Error::InvalidSection(sec) => write!(f, "ERROR: Invalid section specified {}.", sec),
            Error::InvalidGroup(grp) => write!(f, "ERROR: Invalid group specified {}.", grp),
            Error::InvalidComponent(cmp) => {
                write!(f, "ERROR: Invalid component specified {}.", cmp)
            }
            Error::UnsupportedChecksumType(typ) => write!(
                f,
                "ERROR: Unsupported package checksum type specified {}.",
                typ
            ),
            Error::FileNotExist(p) => write!(f, "ERROR: The specified file does not exist: {}", p),
            Error::FileDigestInvalid {
                file: fil,
                cktype: ckt,
                expected: ex,
                actual: act,
            } => write!(
                f,
                "ERROR: The checksum for {} was invalid {}[{} != {}]",
                fil, ckt, act, ex
            ),
        }
    }
}
