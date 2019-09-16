// Needed to bring in Read trait
use std::io::Read;
// Needed to bring in TryInto trait
use std::convert::TryInto;

use std::convert::TryFrom;
use std::path::{Path, PathBuf};

use chrono::{offset, DateTime};
use lazy_static::lazy_static;
use log::{error, info};
use reqwest::header::{CACHE_CONTROL, ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CacheHeaders {
    pub source: Option<String>,
    pub cache_control: Option<CacheControl>,
    pub last_modified: Option<String>,
    pub etag: Option<String>,
}

impl TryFrom<&std::path::Path> for CacheHeaders {
    type Error = Error;

    fn try_from(path: &std::path::Path) -> std::result::Result<Self, Self::Error> {
        serde_json::from_reader(std::io::BufReader::new(
            std::fs::File::open(path).map_err(Self::Error::from)?,
        ))
        .map_err(Self::Error::from)
    }
}

impl CacheHeaders {
    fn new(url: &str) -> Self {
        Self {
            source: Some(url.to_string()),
            cache_control: None,
            last_modified: None,
            etag: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum CacheControl {
    // No caching,
    NoStore,
    // Cache, but check with server to validate
    NoCache,
    // Relative time since original request before invalidation
    // MaxAge http header gets coerced to an Expires variant
    // MaxAge(u32),
    // Fixed time for invalidation
    Expires(DateTime<offset::Utc>),
    // Revalidate with each request
    MustRevalidate,
}

impl TryFrom<&str> for CacheControl {
    type Error = Error;

    fn try_from(content: &str) -> std::result::Result<Self, Self::Error> {
        error!("TODO: implement proper cache-control deserialization ({})", content);
        match content {
            _ => Ok(CacheControl::MustRevalidate),
        }
    }
}

lazy_static! {
    static ref HTTP: reqwest::Client = reqwest::Client::new();
}

pub fn get_cache_dir(sub_path_opt: Option<&Path>) -> Result<PathBuf> {
    let mut dir = dirs::cache_dir().expect("Failed getting local user cache directory");
    dir.push(env!("CARGO_PKG_NAME"));
    if let Some(sub_path) = sub_path_opt {
        dir.push(sub_path);
    }
    Ok(dir)
}

fn get_url_cache_dir(url_str: &str) -> Result<PathBuf> {
    let http_sub_dir = Path::new("http");
    let mut hasher = md5::Context::new();

    hasher.consume(url_str.as_bytes());
    let digest = format!("{:x}", hasher.compute());
    get_cache_dir(Some(&http_sub_dir.join(digest)))
}

fn file_getter(url_str: &str, metadata: &CacheHeaders) -> Result<impl Read> {
    // Construct a request that will either confirm that the cache is valid
    // or provide us with the necessary data
    let mut req = HTTP.get(url_str);
    // Request server verification of cached etag
    if let Some(etag) = &metadata.etag {
        info!("Url has cached version with etag.");
        req = req.header(IF_NONE_MATCH, etag);
    }
    // Request server verification of cached modification date
    if let Some(modified) = &metadata.last_modified {
        info!("Url has cached version with \"last modified\" date.");
        req = req.header(IF_MODIFIED_SINCE, modified.as_str());
    }
    let mut resp = req.send().map_err(Error::from)?;
    let status = resp.status();

    let url_cache = get_url_cache_dir(url_str)?;
    std::fs::create_dir_all(&url_cache)?;

    // New data for us
    if status.is_success() {
        info!(
            "No cache or cache invalid for {}, fetching content...",
            url_str
        );
        // Write data to cache
        let mut out_file = std::io::BufWriter::new(
            std::fs::File::create(url_cache.join("data")).map_err(Error::from)?,
        );

        resp.copy_to(&mut out_file).map_err(Error::from)?;

        // Update cache metadata
        let headers = resp.headers();
        let mut metadata = CacheHeaders::new(url_str);
        metadata.etag = headers
            .get(ETAG)
            .map(|h| std::str::from_utf8(h.as_bytes()).map(|s| s.to_string()))
            .transpose()
            .map_err(Error::from)?;
        metadata.last_modified = headers
            .get(LAST_MODIFIED)
            .map(|h| std::str::from_utf8(h.as_bytes()).map(|s| s.to_string()))
            .transpose()
            .map_err(Error::from)?;
        metadata.cache_control = headers
            .get(CACHE_CONTROL)
            .map(|h| {
                std::str::from_utf8(h.as_bytes())
                    .map_err(Error::from)
                    .and_then(|s| s.try_into())
            })
            .transpose()
            .map_err(Error::from)?;
        let mut out_file = std::io::BufWriter::new(
            std::fs::File::create(url_cache.join("cache")).map_err(Error::from)?,
        );
        serde_json::to_writer_pretty(&mut out_file, &metadata)?;
    } else if status == StatusCode::NOT_MODIFIED {
        // cached data is valid, use that
        info!("Using cached copy of {}...", url_str);
    } else {
        // Some kind of error occurred, for which we can't tell
        // if the cache is valid or not
        error!(
            "Unable to validate cache, nor fetch a new copy of, url {}",
            url_str
        );
        return Err(Error::from(status));
    }

    //  Server reports cached data is still valid
    Ok(std::io::BufReader::new(
        std::fs::File::open(url_cache.join("data")).map_err(Error::from)?,
    ))
}

pub fn cached_get(url_str: &str) -> Result<impl Read> {
    let url_cache = get_url_cache_dir(url_str)?;
    let url_cache_meta_path = url_cache.join("cache");
    let mut metadata = if url_cache_meta_path.exists() {
        CacheHeaders::try_from(Path::new(&url_cache_meta_path)).map_err(Error::from)?
    } else {
        CacheHeaders::new(url_str)
    };

    match metadata
        .cache_control
        .clone()
        .unwrap_or(CacheControl::NoStore)
    {
        // Force ignore cache
        CacheControl::NoStore => {
            info!("Cache disallowed for {}", url_str);
            metadata.etag = None;
            metadata.last_modified = None;
        }
        CacheControl::Expires(dt) if dt >= chrono::Utc::now() => {
            info!("Cache expired for {}", url_str);
            metadata.etag = None;
            metadata.last_modified = None;
        }
        _ => {
            // Allow using cached data if revalidation succeeds
        }
    }
    file_getter(url_str, &metadata)
}
