use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct L2Repo {
    #[serde(skip)]
    pub source: Option<url::Url>,
    pub information: L2Information,
    pub releases: Vec<L2Release>,
}

impl TryFrom<&str> for L2Repo {
    type Error = Error;

    fn try_from(url_str: &str) -> std::result::Result<Self, Self::Error> {
        let mut tmp: Self = serde_json::from_str(
            &reqwest::get(url_str)
                .map_err(Error::from)?
                .text()
                .map_err(Error::from)?,
        )
        .map_err(Error::from)?;
        tmp.source = Some(url::Url::parse(url_str).map_err(Error::from)?);
        Ok(tmp)
    }
}

impl TryFrom<&url::Url> for L2Repo {
    type Error = Error;

    fn try_from(url: &url::Url) -> std::result::Result<Self, Self::Error> {
        Self::try_from(url.as_str())
    }
}

impl L2Repo {
    pub fn releases(&self) -> Vec<String> {
        self.releases.iter().map(|p| p.title.clone()).collect()
    }

    pub fn get_release(&self, title: &str) -> Option<&L2Release> {
        self.releases.iter().find(|p| p.title == title)
    }

    pub fn get_release_url(&self, title: &str) -> Result<url::Url> {
        let release = self
            .get_release(title)
            .ok_or_else(|| Error::InvalidRelease(title.to_owned(), self.releases()))?;
        release.get_url(self.source.as_ref().expect("L2 Repo is missing source field."))
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct L2Information {
    pub title: String,
    pub file_version: String,
    pub file_revision: u8,
    pub server_configuration_build: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct L2Release {
    pub product_category: String,
    #[serde(rename = "targetOS")]
    pub target_os: String,
    pub server_type: Vec<String>,
    pub id: Option<String>,
    pub title: String,
    pub release_version: String,
    pub release_edition: String,
    pub release_edition_message: String,
    pub release_build: String,
    pub release_revision: u8,
    #[serde(rename = "minSDKMVer")]
    pub min_sdkm_ver: String,
    pub release_message: String,
    pub show_in_main_list: Option<bool>,
    pub release_notes: ReleaseNote,
    pub pid_group_id: String,
    pub devzone_program_id: serde_json::Value,
    #[serde(rename = "targetHW")]
    pub target_hw: Vec<String>,
    pub operating_systems_support: Vec<String>,
    pub operating_systems_support_warning: Vec<String>,
    #[serde(rename = "estimateTargetDiskSizeInGB")]
    pub estimate_target_disk_size_in_gb: String,
    pub is_install_on_target_enabled: Option<bool>,
    #[serde(rename = "IntHWSupport")]
    pub int_hw_support: Option<bool>,
    #[serde(rename = "compRepoURL")]
    pub comp_repo_url: Option<String>,
}

impl L2Release {
    pub fn get_url(&self, base: &url::Url) -> Result<url::Url> {
        base.join(self.comp_repo_url.as_ref().ok_or_else(|| Error::L2RepoReleaseMissingUrl(self.title.clone()))?).map_err(Error::from)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseNote {
    pub release_notes_title: String,
    #[serde(rename = "releaseNotesURL")]
    pub release_notes_url: String,
    pub release_notes_tooltip: String,
    pub release_notes_download: bool,
}
