use std::error::Error;
use log::info;
use env_logger;
mod config;
mod tui;
mod restore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // initialize logger
    env_logger::init();
    info!("Starting rustored");

    // load configurations
    let s3_cfg = config::S3Config::load()?;
    let ds_cfg = config::DataStoreConfig::load()?;

    // AWS S3 client
    let aws_conf = aws_config::load_from_env().await;
    let s3_client = aws_sdk_s3::Client::new(&aws_conf);

    // run TUI application
    tui::run_app(s3_client, s3_cfg, ds_cfg).await?;
    Ok(())
}
