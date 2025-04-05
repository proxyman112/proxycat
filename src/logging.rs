use crate::error::{Result, ProxyCatError};
use config::Config;
use env_logger::{Builder, WriteStyle};
use log::{LevelFilter, info, warn};
use std::fs::File;
use std::str::FromStr;

#[derive(Debug, serde::Deserialize)]
pub struct LogConfig {
    pub log_file: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            log_file: "proxycat.log".to_string(),
        }
    }
}

pub fn init_logging_with_level(level: &str) -> Result<()> {
    let config = load_config()?;
    
    let param_level = parse_level(level);
    
    // Create a new builder
    let mut builder = Builder::new();
    
    // Set the log level based only on the command-line parameter
    builder.filter_level(param_level);
    
    // Configure console output
    builder.write_style(WriteStyle::Always);
    
    // Configure file output
    if let Ok(file) = File::create(&config.log_file) {
        builder.target(env_logger::Target::Pipe(Box::new(file)));
    }
    
    // Initialize the logger
    builder.init();
    
    Ok(())
}

fn load_config() -> Result<LogConfig> {
    let config = Config::builder()
        .add_source(config::File::with_name("config").required(false))
        .add_source(config::File::with_name("config.local").required(false))
        .build()
        .map_err(|e| ProxyCatError::Logging(format!("Failed to build config: {}", e)))?;
    
    match config.get::<LogConfig>("logging") {
        Ok(log_config) => {
            info!("Loaded logging configuration from config file");
            Ok(log_config)
        }
        Err(e) => {
            warn!("Could not load logging configuration: {}. Using default values.", e);
            Ok(LogConfig::default())
        }
    }
}

fn parse_level(level_str: &str) -> LevelFilter {
    LevelFilter::from_str(level_str).unwrap_or(LevelFilter::Info)
}

/* // Initialize the logging system with the specified level
pub fn init_logging(level: String) {
    let param_level = parse_level(level.as_str());
    println!("Logging level set to: {}", param_level);
} */