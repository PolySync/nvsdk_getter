use std::convert::TryFrom;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::sdkm;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SdkmConfig {
    #[serde(rename = "mainRepoURL")]
    #[serde(with = "sdkm::url")]
    pub main_repo_url: url::Url,
    #[serde(rename = "PIDServer")]
    pub pid_server: String,
    #[serde(rename = "DevZoneServer")]
    pub dev_zone_server: String,
}

impl TryFrom<&std::path::Path> for SdkmConfig {
    type Error = Error;

    fn try_from(path: &std::path::Path) -> std::result::Result<Self, Self::Error> {
        serde_json::from_reader(std::io::BufReader::new(
            std::fs::File::open(path).map_err(Self::Error::from)?,
        ))
        .map_err(Self::Error::from)
    }
}

lazy_static! {
    // "mainRepoURL": "https://developer.download.nvidia.com/sdkmanager/sdkm-config/main/sdkml1_repo.json"
    static ref MAIN_REPO_URL: url::Url = url::Url::parse("https://developer.download.nvidia.com/sdkmanager/sdkm-config/main/sdkml1_repo.json").expect("Failed parsing default L1 repo url");
}

impl std::default::Default for SdkmConfig {
    fn default() -> Self {
        Self {
            main_repo_url: MAIN_REPO_URL.clone(),
            pid_server: "P".to_string(),
            dev_zone_server: "P".to_string(),
        }
    }
}
