use crate::error::Error;
use crate::sdkm;
use crate::sdkm_l2;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct L3Repo {
    #[serde(skip)]
    pub source: Option<url::Url>,
    pub information: L3Information,
    #[serde(with = "sdkm::url")]
    pub comp_directory: url::Url,
    pub sections: Vec<L3Section>,
    pub groups: HashMap<String, L3Group>,
    pub components: HashMap<String, L3Component>,
}

impl TryFrom<&str> for L3Repo {
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

impl TryFrom<&url::Url> for L3Repo {
    type Error = Error;

    fn try_from(url: &url::Url) -> std::result::Result<Self, Self::Error> {
        Self::try_from(url.as_str())
    }
}

impl L3Repo {
    pub fn sections(&self) -> Vec<String> {
        self.sections.iter().map(|p| p.name.clone()).collect()
    }

    pub fn get_section(&self, name: &str) -> Option<&L3Section> {
        self.sections.iter().find(|p| p.name == name)
    }

    pub fn groups(&self) -> Vec<String> {
        self.groups.keys().map(|g| g.to_owned()).collect()
    }

    pub fn get_group(&self, name: &str) -> Option<&L3Group> {
        self.groups.get(&name.to_owned())
    }

    pub fn components(&self) -> Vec<String> {
        self.components.keys().map(|c| c.to_owned()).collect()
    }

    pub fn get_component(&self, name: &str) -> Option<&L3Component> {
        self.components.get(&name.to_owned())
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct L3Information {
    pub schema_url: String,
    pub schema_version: String,
    pub file_version: String,
    pub release: sdkm_l2::L2Release,
    pub target_access_info: L3TargetAccessInfo,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct L3TargetAccessInfo {
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct L3Section {
    pub id: String,
    pub name: String,
    pub title: String,
    pub selectable: Option<bool>,
    pub selected: Option<bool>,
    pub displayed: Option<bool>,
    pub groups: Vec<String>,
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct L3Group {
    pub id: String,
    pub name: String,
    pub group_type: String,
    pub installed_on: String,
    pub description: String,
    pub flash_message: Option<String>,
    pub versions: Vec<L3GroupVersion>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct L3GroupVersion {
    pub version: String,
    pub components: Vec<L3GroupComponentVersion>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct L3GroupComponentVersion {
    pub id: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct L3Component {
    pub id: String,
    pub name: String,
    pub description: String,
    pub comp_type: String,
    pub is_visible: bool,
    pub license_id: Option<String>,
    pub is_detectable_install: bool,
    pub is_install_path_customizable: bool,
    pub versions: Vec<L3ComponentVersion>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct L3ComponentVersion {
    pub version: String,
    pub operating_systems: Vec<String>,
    #[serde(rename = "installSizeMB")]
    pub install_size_mb: f32,
    pub download_files: Vec<L3ComponentVersionDownloadFile>,
    pub target_ids: Vec<String>,
    pub dependencies: serde_json::Value,
    #[serde(rename = "external_dependencies")]
    pub external_dependencies: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged, rename_all = "camelCase")]
pub enum L3ComponentDependency {
    Plain(String),
    List(Vec<HashMap<String, String>>),
    Map(HashMap<String, String>),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct L3ComponentVersionDownloadFile {
    pub url: String,
    pub file_name: String,
    pub size: u32,
    pub checksum: String,
    pub checksum_type: String,
    pub install_parameters: L3ComponentInstallParameters,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct L3ComponentInstallParameters {
    pub install_type: String,
    pub additional_parameters: L3ComponentAdditionalParameters,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct L3ComponentAdditionalParameters {
    pub packages_info: Option<Vec<HashMap<String, String>>>,
    pub apt_switch: Option<String>,
    pub post_uninstall_commands: Option<Vec<HashMap<String, String>>>,
}
