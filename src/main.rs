use log::info;
use env_logger;
use anyhow::Result;
mod config;
mod tui;
mod restore;

#[tokio::main]
async fn main() -> Result<()> {
    // initialize logger
    env_logger::init();
    info!("Starting rustored");

    // load configurations
    let s3_cfg = config::S3Config::from_env()?;
    let ds_cfg = config::DataStoreConfig::from_env()?;

    // AWS S3 client
    let aws_conf = aws_config::load_from_env().await;
    let s3_client = aws_sdk_s3::Client::new(&aws_conf);

    // run TUI application
    tui::run_app(s3_client, s3_cfg, ds_cfg).await?;
    Ok(())
}
