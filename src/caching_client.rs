#![allow(dead_code)]
/// This module forms the beginnings of a reusable http
/// caching system.  It is able to verify with the server
/// whether the server copy has changed since the file was
/// cached, and serve up the local copy instead.
///
/// TODO:
/// * Allow setting maximum cache size policy
/// * Honor cache-control header
/// * Honor public/private policy
/// * Verify the cached data length against the content-length
///   header in metadata
/// * Pass the url of the request into the cached response object
///   and use that instead of relying on the response url, since
///   redirects and other things could cause cache misses due to
///   request/response name mismatches
// Needed to bring in Read trait
use std::io::Read;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::path::PathBuf;

use chrono::{offset, DateTime};
use encoding_rs::{Encoding, UTF_8};
use log::{debug, info};
use reqwest::header::{CONTENT_TYPE, IF_MODIFIED_SINCE, IF_NONE_MATCH};
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

pub fn url_cache_path(cache_dir: &std::path::Path, url: &str) -> std::path::PathBuf {
    let mut hasher = md5::Context::new();
    hasher.consume(url.as_bytes());
    let digest = format!("{:x}", hasher.compute());
    cache_dir.join(digest)
}

pub fn url_metadata_cache_path(cache_dir: &std::path::Path, url: &str) -> std::path::PathBuf {
    url_cache_path(cache_dir, url).join("metadata")
}

pub fn url_data_cache_path(cache_dir: &std::path::Path, url: &str) -> std::path::PathBuf {
    url_cache_path(cache_dir, url).join("data")
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestMetadata {
    source: String,
    timestamp: DateTime<offset::Utc>,
    response_headers: HashMap<String, Vec<String>>,
}

impl TryFrom<&std::path::Path> for RequestMetadata {
    type Error = Error;

    fn try_from(path: &std::path::Path) -> std::result::Result<Self, Self::Error> {
        debug!("Reading http cache metadata from {:?}", path.to_str());
        serde_json::from_reader(std::io::BufReader::new(
            std::fs::File::open(path).map_err(Self::Error::from)?,
        ))
        .map_err(Self::Error::from)
    }
}

impl From<&reqwest::Response> for RequestMetadata {
    fn from(resp: &reqwest::Response) -> Self {
        debug!("Converting response headers into cache metadata...");
        // Need to convert from HeaderMap (multimap) to a standard hashmap
        // for serialization
        let mut headers_hm: HashMap<String, Vec<String>> = HashMap::new();
        for (key, value) in resp.headers() {
            let key_str = key.as_str();
            let value_str = value.to_str().expect("invalid header value characters.");
            let entry = headers_hm
                .entry(key_str.to_string())
                .or_insert_with(Vec::new);
            entry.push(value_str.to_string());
        }
        Self {
            source: resp.url().as_str().to_string(),
            timestamp: chrono::offset::Utc::now(),
            response_headers: headers_hm,
        }
    }
}

impl Into<reqwest::header::HeaderMap> for RequestMetadata {
    fn into(self) -> reqwest::header::HeaderMap {
        debug!("Converting cache metadata into request headers...");
        let mut out = reqwest::header::HeaderMap::new();
        if let Some(etags) = self.response_headers.get("etag") {
            for etag in etags {
                debug!("Request has etag {}", etag);
                let headerval = reqwest::header::HeaderValue::from_str(etag)
                    .expect("invalid header value characters in etag header.");
                out.append(IF_NONE_MATCH, headerval);
            }
        }
        if let Some(modifieds) = self.response_headers.get("last-modified") {
            for modified in modifieds {
                debug!("Request has modified date {}", modified);
                let headerval = reqwest::header::HeaderValue::from_str(modified)
                    .expect("invalid header value characters in last-modified header.");
                out.append(IF_MODIFIED_SINCE, headerval);
            }
        }
        out
    }
}

pub enum CacheType {
    Public,
    Private,
}

pub struct CachedRequestBuilder {
    cache_type: CacheType,
    cache_dir: std::path::PathBuf,
    inner: reqwest::RequestBuilder,
}

impl CachedRequestBuilder {
    pub fn new(
        cache_type: CacheType,
        cache_dir: &std::path::Path,
        req_build: reqwest::RequestBuilder,
    ) -> Self {
        Self {
            cache_type,
            cache_dir: cache_dir.to_path_buf(),
            inner: req_build,
        }
    }

    pub fn build(self) -> Result<CachedRequest> {
        Ok(CachedRequest::new(
            self.cache_type,
            &self.cache_dir,
            self.inner.build()?,
        ))
    }

    pub fn send(self, client: &reqwest::Client) -> Result<CachedResponse> {
        let req = self.build()?;
        req.send(client)
    }
}

impl std::ops::Deref for CachedRequestBuilder {
    type Target = reqwest::RequestBuilder;

    fn deref(&self) -> &reqwest::RequestBuilder {
        &self.inner
    }
}

pub struct CachedRequest {
    cache_type: CacheType,
    cache_dir: std::path::PathBuf,
    inner: reqwest::Request,
}

impl CachedRequest {
    pub fn new(cache_type: CacheType, cache_dir: &std::path::Path, req: reqwest::Request) -> Self {
        Self {
            cache_type,
            cache_dir: cache_dir.to_path_buf(),
            inner: req,
        }
    }

    pub fn url_cache_path(&self) -> std::path::PathBuf {
        url_cache_path(&self.cache_dir, self.url().as_str())
    }

    pub fn url_data_cache_path(&self) -> std::path::PathBuf {
        url_data_cache_path(&self.cache_dir, self.url().as_str())
    }

    pub fn url_metadata_cache_path(&self) -> std::path::PathBuf {
        url_metadata_cache_path(&self.cache_dir, self.url().as_str())
    }

    pub fn send(mut self, client: &reqwest::Client) -> Result<CachedResponse> {
        // Load cache metadata and convert to headers requesting confirmation
        // that the cached data is valid
        if self.url_metadata_cache_path().exists() {
            let metadata: RequestMetadata =
                RequestMetadata::try_from(self.url_metadata_cache_path().as_path())?;
            let cache_request_headers: reqwest::header::HeaderMap = metadata.into();
            self.inner
                .headers_mut()
                .extend(cache_request_headers.into_iter());
        }

        let builder = CachedResponseBuilder::new(self.cache_type, &self.cache_dir)
            .response(client.execute(self.inner)?);
        builder.build()
    }
}

impl std::ops::Deref for CachedRequest {
    type Target = reqwest::Request;

    fn deref(&self) -> &reqwest::Request {
        &self.inner
    }
}

pub struct CachedResponseBuilder {
    cache_type: CacheType,
    cache_dir: std::path::PathBuf,
    response: Option<reqwest::Response>,
}

impl CachedResponseBuilder {
    pub fn new(cache_type: CacheType, cache_dir: &std::path::Path) -> Self {
        Self {
            cache_type,
            cache_dir: cache_dir.to_path_buf(),
            response: None,
        }
    }

    pub fn response(mut self, resp: reqwest::Response) -> Self {
        self.response = Some(resp);
        self
    }

    pub fn build(self) -> Result<CachedResponse> {
        Ok(CachedResponse {
            cache_type: self.cache_type,
            cache_dir: self.cache_dir.clone(),
            response: self
                .response
                .expect("Cached response builder missing required parameter 'response'."),
        })
    }
}

pub struct CachedResponse {
    cache_type: CacheType,
    cache_dir: std::path::PathBuf,
    response: reqwest::Response,
}

impl std::ops::Deref for CachedResponse {
    type Target = reqwest::Response;

    fn deref(&self) -> &reqwest::Response {
        &self.response
    }
}

impl CachedResponse {
    pub fn url_cache_path(&self) -> std::path::PathBuf {
        url_cache_path(&self.cache_dir, self.url().as_str())
    }

    pub fn url_data_cache_path(&self) -> std::path::PathBuf {
        url_data_cache_path(&self.cache_dir, self.url().as_str())
    }

    pub fn url_metadata_cache_path(&self) -> std::path::PathBuf {
        url_metadata_cache_path(&self.cache_dir, self.url().as_str())
    }

    pub fn cached_text(&mut self) -> Result<String> {
        self.cached_text_with_charset("utf-8")
    }

    pub fn cached_text_with_charset(&mut self, default_encoding: &str) -> Result<String> {
        let content_type = self
            .response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<mime::Mime>().ok());
        let encoding_name = content_type
            .as_ref()
            .and_then(|mime| mime.get_param("charset").map(|charset| charset.as_str()))
            .unwrap_or(default_encoding);
        let encoding = Encoding::for_label(encoding_name.as_bytes()).unwrap_or(UTF_8);
        let mut bytes: Vec<u8> = self
            .response
            .content_length()
            .map(|l| Vec::with_capacity(l as usize))
            .unwrap_or_else(Vec::new);
        let mut reader = self.cached_reader()?;
        reader.read_to_end(&mut bytes)?;
        let (text, _, _) = encoding.decode(&bytes);
        match text {
            std::borrow::Cow::Owned(s) => Ok(s),
            _ => unsafe { Ok(String::from_utf8_unchecked(bytes.to_vec())) },
        }
    }

    pub fn cached_json<T: DeserializeOwned>(&mut self) -> Result<T> {
        let mut bytes: Vec<u8> = self
            .response
            .content_length()
            .map(|l| Vec::with_capacity(l as usize))
            .unwrap_or_else(Vec::new);
        let mut reader = self.cached_reader()?;
        reader.read_to_end(&mut bytes)?;
        serde_json::from_slice(&bytes).map_err(Error::from)
    }

    pub fn cached_copy_to<W: ?Sized>(&mut self, w: &mut W) -> Result<u64>
    where
        W: std::io::Write,
    {
        std::io::copy(&mut self.cached_reader()?, w).map_err(Error::from)
    }

    fn update_cache(&mut self) -> Result<()> {
        // Ensure a cache directory exists
        std::fs::create_dir_all(&self.url_cache_path())?;

        // Write data to cache
        debug!(
            "Caching {} to {:?}",
            self.url(),
            self.url_data_cache_path().to_str()
        );
        let mut out_file = std::io::BufWriter::new(
            std::fs::File::create(self.url_data_cache_path()).map_err(Error::from)?,
        );

        self.response.copy_to(&mut out_file).map_err(Error::from)?;

        // Write metadata to cache
        debug!(
            "Caching {} metadata to {:?}",
            self.url(),
            self.url_metadata_cache_path().to_str()
        );
        let req_metadata = RequestMetadata::from(&self.response);
        let mut out_file = std::io::BufWriter::new(
            std::fs::File::create(self.url_metadata_cache_path()).map_err(Error::from)?,
        );
        serde_json::to_writer_pretty(&mut out_file, &req_metadata).map_err(Error::from)
    }

    pub fn cached_file_path(&mut self) -> Result<PathBuf> {
        // Check the response for information about whether our cached data
        // is valid
        let status = self.response.status();

        // New data for us
        if status.is_success() {
            info!("Downloading {} into the cache...", self.response.url());
            self.update_cache()?;
        } else if status == StatusCode::NOT_MODIFIED {
            // cached data is valid, use that
            info!("Using cached copy of {}", self.response.url());
        } else {
            // Some kind of error occurred, for which we can't tell
            // if the cache is valid or not
            return Err(Error::from(status));
        }

        // There should now be a locally cached version of the requested url
        Ok(self.url_data_cache_path())
    }

    pub fn cached_reader(&mut self) -> Result<impl Read> {
        Ok(std::io::BufReader::new(
            std::fs::File::open(self.cached_file_path()?).map_err(Error::from)?,
        ))
    }
}
