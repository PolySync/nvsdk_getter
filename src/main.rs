use log::{debug, error};
use std::collections::HashSet;
use std::convert::TryFrom;
use std::path::PathBuf;
use structopt::StructOpt;

mod error;
use error::{Error, Result};
mod sdkm;
mod sdkm_config;
use sdkm_config::SdkmConfig;
mod sdkm_l1;
use sdkm_l1::L1Repo;
mod sdkm_l2;
use sdkm_l2::L2Repo;
mod sdkm_l3;
use sdkm_l3::L3Repo;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Enable debugging output
    #[structopt(short = "g", long)]
    debug: bool,

    /// Verbose mode, repeat to increase verbosity
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    /// Quiet mode, silence output - this supercedes other
    /// output control options
    #[structopt(short, long)]
    quiet: bool,

    /// Path to the sdkm_config.json file from the SDKManager
    #[structopt(short = "c", long, parse(from_os_str))]
    sdkm_config: PathBuf,

    /// Product Category, leave unspecified to see a list options
    #[structopt(short, long)]
    product_category: Option<String>,

    /// Target OS, leave unspecified to see a list of options
    #[structopt(short, long)]
    target_os: Option<String>,

    /// Product Release, leave unspecified to see a list of options
    #[structopt(short, long)]
    release: Option<String>,

    /// List package sections, groups, and components
    #[structopt(long)]
    list: bool,

    /// List package sections
    #[structopt(long)]
    list_sections: bool,

    /// Show section details
    #[structopt(long)]
    section_info: Option<String>,

    /// Package Section to fetch, repeat to enable multiple sections
    #[structopt(short, long)]
    section: Vec<String>,

    /// List package groups
    #[structopt(long)]
    list_groups: bool,

    /// Show group details
    #[structopt(long)]
    group_info: Option<String>,

    /// Package Group to fetch, repeat to enable multiple groups
    #[structopt(short = "G", long)]
    group: Vec<String>,

    /// List package components
    #[structopt(long)]
    list_components: bool,

    /// Show component details
    #[structopt(long)]
    component_info: Option<String>,

    /// Package Component to fetch, repeat to enable multiple components
    #[structopt(short = "C", long)]
    component: Vec<String>,
}

fn get_log_level(opt: &Opt) -> flexi_logger::LevelFilter {
    if opt.quiet {
        flexi_logger::LevelFilter::Off
    } else if opt.debug {
        match opt.verbose {
            0 => flexi_logger::LevelFilter::Debug,
            _ => flexi_logger::LevelFilter::Trace,
        }
    } else {
        match opt.verbose {
            0 => flexi_logger::LevelFilter::Error,
            1 => flexi_logger::LevelFilter::Warn,
            _ => flexi_logger::LevelFilter::Info,
        }
    }
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    flexi_logger::Logger::with(
        flexi_logger::LogSpecification::default(get_log_level(&opt)).build(),
    )
    .start()
    .map_err(Error::from)?;
    debug!("Parsed args: {:?}", opt);

    let config = SdkmConfig::try_from(opt.sdkm_config.as_path())?;

    debug!("SDKManager Config: {:?}", config);

    let l1repo = L1Repo::try_from(&config.main_repo_url)?;
    debug!("L1 Repo: {:?}", l1repo);

    let req_product_category = opt
        .product_category
        .ok_or_else(|| Error::MissingProductCategory(l1repo.product_categories()))?;

    let product_category = l1repo
        .get_product_category(&req_product_category)
        .ok_or_else(|| {
            Error::InvalidProductCategory(req_product_category.clone(), l1repo.product_categories())
        })?;
    debug!("Product Category: {:?}", product_category);

    let req_target_os = opt
        .target_os
        .ok_or_else(|| Error::MissingTargetOS(product_category.product_lines()))?;
    let product_line = product_category
        .get_product_line(&req_target_os)
        .ok_or_else(|| {
            Error::InvalidTargetOS(req_target_os.clone(), product_category.product_lines())
        })?;
    debug!("Target OS: {:?}", product_line);

    let l2_rel_url = l1repo.get_product_url(&req_product_category, &req_target_os)?;
    debug!("l2_rel_url: {}", l2_rel_url);
    let l2repo = L2Repo::try_from(&l2_rel_url)?;
    debug!("L2 Repo: {:?}", l2repo);

    let req_release = opt
        .release
        .ok_or_else(|| Error::MissingRelease(l2repo.releases()))?;

    let release = l2repo
        .get_release(&req_release)
        .ok_or_else(|| Error::InvalidRelease(req_release.clone(), l2repo.releases()))?;
    debug!("Release: {:?}", release);
    let l3_url = l2repo.get_release_url(&req_release)?;
    debug!("l3_url: {}", l3_url);

    let l3repo = L3Repo::try_from(&l3_url)?;
    debug!("L3 Repo: {:?}", l3repo);

    if opt.list || opt.list_sections {
        println!("Package sections:");
        for section in l3repo.sections() {
            println!("\t{}", section);
        }
    }

    if let Some(section_id) = opt.section_info {
        let section = l3repo
            .get_section(&section_id)
            .ok_or_else(|| Error::InvalidSection(section_id))?;
        println!("{:?}", section);
    }

    if opt.list || opt.list_groups {
        println!("Package groups:");
        for group in l3repo.groups() {
            println!("\t{}", group);
        }
    }

    if let Some(group_id) = opt.group_info {
        let group = l3repo
            .get_group(&group_id)
            .ok_or_else(|| Error::InvalidGroup(group_id))?;
        println!("{:?}", group);
    }

    if opt.list || opt.list_components {
        println!("Package components:");
        for component in l3repo.components() {
            println!("\t{}", component);
        }
    }

    if let Some(component_id) = opt.component_info {
        let component = l3repo
            .get_component(&component_id)
            .ok_or_else(|| Error::InvalidComponent(component_id))?;
        println!("{:?}", component);
    }

    let mut component_ids: HashSet<String> = opt.component.into_iter().collect();
    for section in opt.section {
        component_ids.extend(l3repo.get_components_for_section(&section).into_iter());
    }
    for group in opt.group {
        component_ids.extend(l3repo.get_components_for_group(&group).into_iter());
    }
    error!("NOT IMPLEMENTED: packages to fetch... {:?}", component_ids);

    Ok(())
}
