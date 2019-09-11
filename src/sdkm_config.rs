use crate::error::Error;
use crate::sdkm;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

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
