use log::{error, info, warn, debug};
use std::collections::HashSet;
use std::io::BufRead;
use std::path::Path;
use std::convert::TryInto;
use structopt::StructOpt;

use crate::error::{Error, Result};
use crate::sdkm_l3::L3Repo;

#[derive(Debug, StructOpt)]
pub enum Action {
    /// Give information about package sections, groups, and components
    Show {
        /// Package section, repeat to specify multiple sections
        #[structopt(short, long)]
        section: Vec<String>,

        /// Package group, repeat to specify multiple groups
        #[structopt(short, long)]
        group: Vec<String>,

        /// Package component, repeat to specify multiple components
        #[structopt(short, long)]
        component: Vec<String>,
    },
    /// Fetch packages belonging to specified section, group, or component
    Fetch {
        /// Package section, repeat to specify multiple sections
        #[structopt(short, long)]
        section: Vec<String>,

        /// Package group, repeat to specify multiple groups
        #[structopt(short, long)]
        group: Vec<String>,

        /// Package component, repeat to specify multiple components
        #[structopt(short, long)]
        component: Vec<String>,
    },
    /// Verify local cache of packages belonging to specified section, group, or component
    Verify {
        /// Package section, repeat to specify multiple sections
        #[structopt(short, long)]
        section: Vec<String>,

        /// Package group, repeat to specify multiple groups
        #[structopt(short, long)]
        group: Vec<String>,

        /// Package component, repeat to specify multiple components
        #[structopt(short, long)]
        component: Vec<String>,
    },
}

impl Action {
    pub fn get_sections(&self) -> &Vec<String> {
        match self {
            Action::Show { section, .. } => &section,
            Action::Fetch { section, .. } => &section,
            Action::Verify { section, .. } => &section,
        }
    }

    pub fn get_groups(&self) -> &Vec<String> {
        match self {
            Action::Show { group, .. } => &group,
            Action::Fetch { group, .. } => &group,
            Action::Verify { group, .. } => &group,
        }
    }

    pub fn get_components(&self) -> &Vec<String> {
        match self {
            Action::Show { component, .. } => &component,
            Action::Fetch { component, .. } => &component,
            Action::Verify { component, .. } => &component,
        }
    }
}

fn get_component_ids(l3repo: &L3Repo, action_data: &Action) -> HashSet<String> {
    let mut component_ids: HashSet<String> = action_data
        .get_components()
        .iter()
        .map(|c| c.to_string())
        .collect();
    for section in action_data.get_sections() {
        component_ids.extend(l3repo.get_components_for_section(&section).into_iter());
    }
    for group in action_data.get_groups() {
        component_ids.extend(l3repo.get_components_for_group(&group).into_iter());
    }
    component_ids
}

pub fn show(l3repo: &L3Repo, action_data: &Action) -> Result<()> {
    if action_data.get_sections().is_empty()
        && action_data.get_groups().is_empty()
        && action_data.get_components().is_empty()
    {
        println!("Package sections:");
        for section_id in l3repo.sections() {
            println!("\t{}", section_id);
        }

        println!("Package groups:");
        for group_id in l3repo.groups() {
            println!("\t{}", group_id);
        }

        println!("Package components:");
        for component_id in l3repo.components() {
            println!("\t{}", component_id);
        }
    }

    for section_id in action_data.get_sections() {
        let section = l3repo
            .get_section(section_id)
            .ok_or_else(|| Error::InvalidSection(section_id.to_string()))?;
        println!(
            "Section {}: {}[{}]",
            section.id, section.title, section.name
        );
        for group_id in &section.groups {
            println!("\tChild group: {}", group_id);
        }
    }

    for group_id in action_data.get_groups() {
        let group = l3repo
            .get_group(group_id)
            .ok_or_else(|| Error::InvalidGroup(group_id.to_string()))?;
        println!("Group {}: {}[{}]", group.id, group.name, group.installed_on);
        println!("\tDescription: {}", group.description);
        for version in &group.versions {
            println!("\tVersion {} components:", version.version);
            for component in &version.components {
                println!("\t\t{}", component.id);
            }
        }
    }

    for component_id in action_data.get_components() {
        let component = l3repo
            .get_component(component_id)
            .ok_or_else(|| Error::InvalidComponent(component_id.to_string()))?;
        println!(
            "Component {}: {}[{}]",
            component.id, component.name, component.comp_type
        );
        println!("\tDescription: {}", component.description);
        for version in &component.versions {
            println!("\tVersion {}:", version.version);
            println!("\t\tInstall size: {} MB", version.install_size_mb);
            for os in &version.operating_systems {
                println!("\t\tSupported OS: {}", os);
            }
            for target_id in &version.target_ids {
                println!("\t\tSupported HW: {}", target_id);
            }
            for file in &version.download_files {
                println!("\t\tPackage {}", file.file_name)
            }
        }
    }
    Ok(())
}

pub fn fetch(l3repo: &L3Repo, action_data: &Action, cache_dir: &Path) -> Result<()> {
    let client = reqwest::Client::new();
    debug!("Creating cache directory {} (if it doesn't already exist)", cache_dir.to_string_lossy().to_string());
    std::fs::create_dir_all(cache_dir).map_err(Error::from)?;
    for component_id in get_component_ids(l3repo, action_data) {
        let component = l3repo
            .get_component(&component_id)
            .ok_or_else(|| Error::InvalidComponent(component_id.to_string()))?;
        if let Some(component_ver) = component.versions.first() {
            info!(
                "Retrieving {} version {}.",
                component_id, component_ver.version
            );
            warn!("Other versions may be available, but selecting them is not yet supported.");
            for file in &component_ver.download_files {
                let local_filename = cache_dir.join(file.file_name.clone());
                let remote_file_url = l3repo
                    .source
                    .as_ref()
                    .expect("Source not set on l3 repo!")
                    .join(&file.url)
                    .map_err(Error::from)?;
                let mut out_file = std::io::BufWriter::new(
                    std::fs::File::create(local_filename).map_err(Error::from)?,
                );
                info!(
                    "Retrieving {} package {} into {}...",
                    component_id,
                    file.file_name,
                    cache_dir.display()
                );
                let mut resp = client
                    .get(remote_file_url.as_str())
                    .send()
                    .map_err(Error::from)?;
                resp.copy_to(&mut out_file).map_err(Error::from)?;
            }
        }
    }
    Ok(())
}

fn validate_file(filename: &Path, checksum_type: &str, checksum: &str) -> Result<()> {
    if !filename.exists() {
        Err(Error::FileNotExist(filename.to_string_lossy().to_string()))?;
    }

    info!("Verifying file checksum...");
    let file_meta = std::fs::metadata(filename)?;
    let mut in_file = std::io::BufReader::new(std::fs::File::open(filename).map_err(Error::from)?);
    match checksum_type {
        "md5" => {
            let mut hasher = md5::Context::new();
            let mut count = 0;
            let bar = indicatif::ProgressBar::new(file_meta.len());
            while !in_file.fill_buf().map_err(Error::from)?.is_empty() {
                let buf_len = in_file.buffer().len();
                debug!("Updating checksum from {} bytes...", buf_len);
                hasher.consume(in_file.buffer());
                in_file.consume(buf_len);
                bar.inc(buf_len.try_into().unwrap());
            }
            let digest = hasher.compute();
            let digest_str = format!("{:x}", digest);
            if digest_str != checksum {
                Err(Error::FileDigestInvalid {
                    file: filename.to_string_lossy().to_string(),
                    cktype: checksum_type.to_string(),
                    expected: checksum.to_string(),
                    actual: digest_str,
                })?;
            }
        }
        _ => Err(Error::UnsupportedChecksumType(checksum_type.to_owned()))?,
    }
    Ok(())
}

pub fn verify(l3repo: &L3Repo, action_data: &Action, cache_dir: &Path) -> Result<()> {
    for component_id in get_component_ids(l3repo, action_data) {
        let component = l3repo
            .get_component(&component_id)
            .ok_or_else(|| Error::InvalidComponent(component_id))?;

        for version in &component.versions {
            for file in &version.download_files {
                let local_filename = cache_dir.join(file.file_name.clone());
                if let Err(e) = validate_file(&local_filename, &file.checksum_type, &file.checksum)
                {
                    match e {
                        Error::FileDigestInvalid {
                            file: f,
                            cktype: ct,
                            expected: c,
                            actual: d,
                        } => error!("INVALID DIGEST: {}[{}] {} != {}", f, ct, d, c),
                        Error::FileNotExist(f) => error!("MISSING FILE:   {} does not exist", f),
                        _ => Err(e)?,
                    }
                } else {
                    info!("VALID:   {}", local_filename.to_string_lossy().to_string());
                }
            }
        }
    }
    Ok(())
}
