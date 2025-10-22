mod config;
mod logging;
mod metrics;
mod unifi;

use anyhow::Result;
use axum::{Router, routing::get};
use std::sync::Arc;
use tokio::time::{Duration, interval};

use log::{error, info};
use std::env;

use crate::config::Config;
use crate::logging::setup_logging;
use crate::metrics::metrics_handler;
use crate::unifi::{NetworkClient, ProtectClient, UnifiCache};

#[derive(Clone)]
struct AppState {
	bearer_token: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args: Vec<String> = env::args().collect();

	if args.iter().any(|a| a == "--version" || a == "-v") {
		println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
		return Ok(());
	}

	let config_path = env::args()
		.nth(1)
		.unwrap_or_else(|| "config.toml".to_string());

	let config = Config::from_file(&config_path)?;

	setup_logging(&config.logging)?;

	info!("UnifiMetrics started with config: {}", config_path);

	info!(
		"Metrics API enabled on port {} (format: OpenMetrics)",
		config.server.port,
	);

	if config.server.bearer_token.is_some() {
		info!("Bearer token authentication enabled for metrics endpoint");
	}

	// Create HTTP client with certificate validation disabled
	let client = reqwest::Client::builder()
		.danger_accept_invalid_certs(true)
		.timeout(Duration::from_secs(5))
		.build()?;

	// Create UniFi clients
	let network_client = Arc::new(NetworkClient::new(
		client.clone(),
		config.unifi.ip.clone(),
		config.unifi.api_token.clone(),
	));

	let protect_client = Arc::new(ProtectClient::new(
		client.clone(),
		config.unifi.ip.clone(),
		config.unifi.api_token.clone(),
	));

	// Initialize cache
	let cache = Arc::new(UnifiCache::new());

	// Initialize network monitoring if enabled
	if config.monitoring.network_devices {
		info!("Initializing network device monitoring");
		match network_client.initialize_sites(&cache).await {
			Ok(_) => info!("Network sites and devices loaded successfully"),
			Err(e) => error!("Failed to initialize network monitoring: {}", e),
		}
	}

	// Clone for the polling task
	let poll_config = config.clone();
	let poll_network_client = network_client.clone();
	let poll_protect_client = protect_client.clone();
	let poll_cache = cache.clone();

	// Start background polling task
	tokio::spawn(async move {
		let mut ticker = interval(Duration::from_secs(poll_config.unifi.poll_interval));
		ticker.tick().await; // Skip first immediate tick

		loop {
			ticker.tick().await;

			// Poll network devices
			if poll_config.monitoring.network_devices {
				info!("Polling network device statistics");
				if let Err(e) = poll_network_client.poll_statistics(&poll_cache).await {
					error!("Failed to poll network statistics: {}", e);
				}
			}

			// Poll protect sensors
			if poll_config.monitoring.protect_sensors {
				info!("Polling protect sensor data");
				if let Err(e) = poll_protect_client.poll_sensors().await {
					error!("Failed to poll protect sensors: {}", e);
				}
			}
		}
	});

	let state = AppState {
		bearer_token: config.server.bearer_token.clone(),
	};

	// Build the application router
	let app = Router::new()
		.route("/metrics", get(metrics_handler))
		.route("/health", get(health_handler))
		.with_state(state);

	// Start the server
	info!(
		"Starting HTTP server on {}:{}",
		config.server.bind_address, config.server.port
	);

	let listener = tokio::net::TcpListener::bind(format!(
		"{}:{}",
		config.server.bind_address, config.server.port
	))
	.await
	.expect("Failed to bind to address");

	axum::serve(listener, app).await?;

	Ok(())
}

async fn health_handler() -> &'static str {
	"OK"
}
