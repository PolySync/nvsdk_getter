use crate::error::{Error, Result};
use log::{error, warn};
use std::collections::HashSet;
use structopt::StructOpt;

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

// TODO: need additional parameter for target dir hint
pub fn fetch(l3repo: &L3Repo, action_data: &Action) -> Result<()> {
    let component_ids = get_component_ids(l3repo, action_data);
    let component_urls: HashSet<url::Url> = component_ids
        .into_iter()
        .flat_map(|id| l3repo.get_component_urls(&id).into_iter())
        .collect();

    if component_urls.is_empty() {
        warn!("Fetch: Nothing to do!");
    }

    for component_url in &component_urls {
        error!("TODO: fetch {}", component_url);
    }
    Ok(())
}

// TODO: need additional parameter for target dir hint
pub fn verify(l3repo: &L3Repo, action_data: &Action) -> Result<()> {
    for component_id in get_component_ids(l3repo, action_data) {
        let component = l3repo
            .get_component(&component_id)
            .ok_or_else(|| Error::InvalidComponent(component_id))?;

        for version in &component.versions {
            for file in &version.download_files {
                println!(
                    "TODO: verify {}({}[{}])",
                    file.file_name, file.checksum_type, file.checksum
                )
            }
        }
    }
    Ok(())
}
