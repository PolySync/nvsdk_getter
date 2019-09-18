use std::convert::TryFrom;
use std::path::{Path, PathBuf};

use human_panic::setup_panic;
use log::debug;
use structopt::StructOpt;

mod error;
use error::{Error, Result};
mod cache;
mod sdkm;
mod sdkm_config;
use sdkm_config::SdkmConfig;
mod sdkm_l1;
use sdkm_l1::L1Repo;
mod sdkm_l2;
use sdkm_l2::L2Repo;
mod sdkm_l3;
use sdkm_l3::L3Repo;
mod actions;
use actions::{fetch, show, verify, Action};
mod caching_client;

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
    sdkm_config: Option<PathBuf>,

    /// Product Category, leave unspecified to see a list options
    #[structopt(short, long)]
    product_category: Option<String>,

    /// Target OS, leave unspecified to see a list of options
    #[structopt(short, long)]
    target_os: Option<String>,

    /// Product Release, leave unspecified to see a list of options
    #[structopt(short, long)]
    release: Option<String>,

    /// Cache directory where local copies of packages are kept
    /// Default is <cache_dir>/nv_getter/<Category>/<TargetOS>/<Release>/
    #[structopt(short = "d", long, parse(from_os_str))]
    cache_dir: Option<PathBuf>,

    /// Software section, group, and component actions
    #[structopt(subcommand)]
    action: Action,
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
    setup_panic!();
    let opt = Opt::from_args();
    flexi_logger::Logger::with(
        flexi_logger::LogSpecification::default(flexi_logger::LevelFilter::Error)
            .module(env!("CARGO_PKG_NAME"), get_log_level(&opt))
            .build(),
    )
    .start()
    .map_err(Error::from)?;
    debug!("Parsed args: {:?}", opt);

    let config = opt
        .sdkm_config
        .map(|c| SdkmConfig::try_from(c.as_path()))
        .transpose()?
        .unwrap_or_else(SdkmConfig::default);

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

    // Default is ~/.cache/nv_getter/<Category>/<TargetOS>/<Release>/
    let cache_dir: PathBuf = opt.cache_dir.unwrap_or_else(|| {
        let dir_str = format!("{}/{}/{}", req_product_category, req_target_os, req_release);
        let dir = Path::new(&dir_str);
        cache::get_cache_dir(Some(&dir))
    });
    std::fs::create_dir_all(&cache_dir)?;
    match &opt.action {
        Action::Show { .. } => show(&l3repo, &opt.action)?,
        Action::Fetch { .. } => fetch(&l3repo, &opt.action, &cache_dir)?,
        Action::Verify { .. } => verify(&l3repo, &opt.action, &cache_dir)?,
    }

    Ok(())
}
