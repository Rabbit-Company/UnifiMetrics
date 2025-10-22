use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
	pub unifi: UnifiConfig,
	pub monitoring: MonitoringConfig,
	pub server: ServerConfig,
	pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UnifiConfig {
	pub ip: String,
	pub api_token: String,
	pub poll_interval: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MonitoringConfig {
	pub network_devices: bool,
	pub protect_sensors: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
	pub bind_address: String,
	pub port: u16,
	pub bearer_token: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
	pub log_file: Option<String>,
	pub log_level: String,
}

impl Config {
	pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
		let config_str = fs::read_to_string(path)?;
		let config: Config = toml::from_str(&config_str)?;
		Ok(config)
	}
}

impl Default for Config {
	fn default() -> Self {
		Config {
			unifi: UnifiConfig {
				ip: "10.0.0.1".to_string(),
				api_token: "".to_string(),
				poll_interval: 30,
			},
			monitoring: MonitoringConfig {
				network_devices: true,
				protect_sensors: true,
			},
			server: ServerConfig {
				bind_address: "0.0.0.0".to_string(),
				port: 9090,
				bearer_token: None,
			},
			logging: LoggingConfig {
				log_file: None,
				log_level: "info".to_string(),
			},
		}
	}
}
