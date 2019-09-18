// Needed to bring in Read trait
use std::io::Read;

use std::path::{Path, PathBuf};

use lazy_static::lazy_static;

use crate::caching_client::{CacheType, CachedRequestBuilder};
use crate::error::Result;

pub fn get_cache_dir(sub_path_opt: Option<&Path>) -> PathBuf {
    let mut dir = dirs::cache_dir().expect("Failed getting local user cache directory");
    dir.push(env!("CARGO_PKG_NAME"));
    if let Some(sub_path) = sub_path_opt {
        dir.push(sub_path);
    }
    dir
}

lazy_static! {
    static ref HTTP: reqwest::Client = reqwest::Client::new();
}

pub fn cached_get_path(url_str: &str) -> Result<PathBuf> {
    let req = HTTP.get(url_str);
    let mut c_resp = CachedRequestBuilder::new(
        CacheType::Private,
        &get_cache_dir(Some(Path::new("http_cache"))),
        req,
    )
    .send(&HTTP)?;
    c_resp.cached_file_path()
}

pub fn cached_get_reader(url_str: &str) -> Result<impl Read> {
    let req = HTTP.get(url_str);
    let mut c_resp = CachedRequestBuilder::new(
        CacheType::Private,
        &get_cache_dir(Some(Path::new("http_cache"))),
        req,
    )
    .send(&HTTP)?;
    c_resp.cached_reader()
}
