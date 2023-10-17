use std::fs::OpenOptions;

use clap::Parser;
use cmd::models::CryptoNautConfig;
use config::Config;
use tracing::error;
use tracing_subscriber::{filter::LevelFilter, EnvFilter};

use crate::cmd::models::CryptoNautError;

mod cmd;

#[tokio::main]
async fn main() -> Result<(), CryptoNautError> {


    let config = Config::builder()
    .add_source(config::File::with_name("src/config.yaml").required(false))
    .add_source(config::File::with_name("/etc/dracoon/config.yaml").required(false))
    .build()
    .expect("Failed to load configuration")
    .try_deserialize::<CryptoNautConfig>()
    .expect("Failed to deserialize configuration");

    // parse command line arguments
    let cmd = cmd::models::CryptoNaut::parse();

    // set up logging level
    let env_filter = if cmd.debug {
        EnvFilter::from_default_env().add_directive(LevelFilter::DEBUG.into())
    } else {
        EnvFilter::from_default_env().add_directive(LevelFilter::INFO.into())
    };

    // set up logging file
    let log_file_path = cmd.log_file_path.unwrap_or("cryptonaut.log".to_string());

    let log_file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(log_file_path)
        .map_err(|e| {
            error!("Failed to create or open log file: {}", e);
            CryptoNautError::LogFileCreationFailed
        })?;

    // set up logging format
    let log_format = tracing_subscriber::fmt::format()
        .with_level(true)
        .with_thread_names(false)
        .with_target(true)
        .with_ansi(false)
        .compact();

    // initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .event_format(log_format)
        .with_writer(std::sync::Mutex::new(log_file))
        .init();

    // // run key distribution
    cmd::distribute_missing_keys(cmd.target_path, &config)
        .await
        .map_err(|err| {
            error!("Error: {}", err);
            err
        })?;

    Ok(())
}
