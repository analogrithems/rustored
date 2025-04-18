use log::info;
use env_logger;
use anyhow::Result;
use clap::Parser;
mod cli;
mod config;
mod tui;
mod restore;

use crate::cli::Opt;

#[tokio::main]
async fn main() -> Result<()> {
    // initialize logger
    env_logger::init();
    info!("Starting rustored");

    // load configurations from CLI args (with env fallbacks)
    let opt = Opt::parse();
    let s3_cfg = config::S3Config::from_opt(&opt);
    let ds_cfg = config::DataStoreConfig::from_opt(&opt);

    // run TUI application, including splash screen
    tui::run_app(s3_cfg, ds_cfg, opt.splash_image.clone()).await?;
    Ok(())
}
