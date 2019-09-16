// Needed to bring in Read trait
use std::io::Read;

use std::convert::TryFrom;

use serde::{Deserialize, Serialize};

use crate::cache;
use crate::error::{Error, Result};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct L1Repo {
    #[serde(skip)]
    pub source: Option<url::Url>,
    pub information: L1Information,
    pub product_categories: Vec<L1ProductCategory>,
}

impl TryFrom<&str> for L1Repo {
    type Error = Error;

    fn try_from(url_str: &str) -> std::result::Result<Self, Self::Error> {
        let mut url_data = String::new();
        let mut in_file = cache::cached_get_reader(url_str)?;
        in_file
            .read_to_string(&mut url_data)
            .map_err(Self::Error::from)?;
        let mut tmp: Self = serde_json::from_str(&url_data).map_err(Self::Error::from)?;
        tmp.source = Some(url::Url::parse(url_str).map_err(Self::Error::from)?);
        Ok(tmp)
    }
}

impl TryFrom<&url::Url> for L1Repo {
    type Error = Error;

    fn try_from(url: &url::Url) -> std::result::Result<Self, Self::Error> {
        Self::try_from(url.as_str())
    }
}

impl L1Repo {
    pub fn product_categories(&self) -> Vec<String> {
        self.product_categories
            .iter()
            .map(|p| p.category_name.clone())
            .collect()
    }

    pub fn get_product_category(&self, product_category: &str) -> Option<&L1ProductCategory> {
        self.product_categories
            .iter()
            .find(|p| p.category_name == product_category)
    }

    pub fn get_product_url(&self, product_category: &str, target_os: &str) -> Result<url::Url> {
        let product_category = self.get_product_category(product_category).ok_or_else(|| {
            Error::InvalidProductCategory(product_category.to_owned(), self.product_categories())
        })?;
        product_category.get_product_line_url(
            &self
                .source
                .as_ref()
                .expect("L1 Repo is missing source field."),
            target_os,
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct L1Information {
    pub title: String,
    pub version: String,
    pub revision: u8,
    pub server_configuration_build: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct L1ProductCategory {
    pub category_name: String,
    pub product_lines: Vec<L1ProductLine>,
}

impl L1ProductCategory {
    pub fn product_lines(&self) -> Vec<String> {
        self.product_lines
            .iter()
            .map(|p| p.target_os.clone())
            .collect()
    }

    pub fn get_product_line(&self, target_os: &str) -> Option<&L1ProductLine> {
        self.product_lines.iter().find(|p| p.target_os == target_os)
    }

    pub fn get_product_line_url(&self, base: &url::Url, target_os: &str) -> Result<url::Url> {
        let product_line = self
            .get_product_line(target_os)
            .ok_or_else(|| Error::InvalidTargetOS(target_os.to_owned(), self.product_lines()))?;
        product_line.get_url(base)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct L1ProductLine {
    #[serde(rename = "targetOS")]
    pub target_os: String,
    pub target_type: String,
    pub server_type: Vec<String>,
    #[serde(rename = "releasesIndexURL")]
    pub releases_index_url: String,
}

impl L1ProductLine {
    pub fn get_url(&self, base: &url::Url) -> Result<url::Url> {
        base.join(&self.releases_index_url).map_err(Error::from)
    }
}
