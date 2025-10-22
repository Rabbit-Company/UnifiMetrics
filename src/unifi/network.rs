use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use reqwest::Client;
use std::time::Instant;

use super::cache::UnifiCache;
use super::models::{ApiError, DeviceStatistics, DevicesResponse, SitesResponse};
use crate::metrics::{update_device_metrics, update_poll_metrics};

pub struct NetworkClient {
	client: Client,
	base_url: String,
	api_token: String,
}

impl NetworkClient {
	pub fn new(client: Client, ip: String, api_token: String) -> Self {
		Self {
			client,
			base_url: format!("https://{}/proxy/network/integration/v1", ip),
			api_token,
		}
	}

	pub async fn initialize_sites(&self, cache: &UnifiCache) -> Result<()> {
		// Fetch all sites
		let sites = self.get_sites().await?;
		info!("Found {} sites", sites.data.len());

		// Update cache with sites
		cache.update_sites(sites.data.clone());

		// Fetch devices for each site
		for site in &sites.data {
			info!("Fetching devices for site: {} ({})", site.name, site.id);
			match self.get_devices(&site.id).await {
				Ok(devices) => {
					info!("Found {} devices in site {}", devices.data.len(), site.name);
					cache.update_devices(&site.id, devices.data);
				}
				Err(e) => {
					error!("Failed to fetch devices for site {}: {}", site.name, e);
				}
			}
		}

		Ok(())
	}

	pub async fn poll_statistics(&self, cache: &UnifiCache) -> Result<()> {
		let start = Instant::now();
		let mut success = true;

		let sites = cache.get_sites();

		for site in sites {
			for (device_id, device) in &site.devices {
				debug!(
					"Polling statistics for device {} ({}) in site {}",
					device.name, device_id, site.name
				);

				match self.get_device_statistics(&site.id, device_id).await {
					Ok(stats) => {
						// Determine device state
						let state_value = if device.state.to_uppercase() == "ONLINE" {
							1
						} else {
							0
						};

						// Update metrics
						update_device_metrics(
							&site.id,
							&site.name,
							device_id,
							&device.name,
							&device.model,
							device.ip_address.as_deref().unwrap_or("unknown"),
							stats.cpu_utilization_pct,
							stats.memory_utilization_pct,
							stats.uplink.as_ref().and_then(|u| u.tx_rate_bps),
							stats.uplink.as_ref().and_then(|u| u.rx_rate_bps),
							state_value,
						);
					}
					Err(e) => {
						warn!(
							"Failed to fetch statistics for device {} in site {}: {}",
							device.name, site.name, e
						);
						success = false;
					}
				}
			}
		}

		let duration = start.elapsed().as_secs_f64();
		update_poll_metrics("network", success, duration);

		Ok(())
	}

	async fn get_sites(&self) -> Result<SitesResponse> {
		let url = format!("{}/sites?limit=25", self.base_url);

		let response = self
			.client
			.get(&url)
			.header("X-API-KEY", &self.api_token)
			.header("Accept", "application/json")
			.send()
			.await
			.context("Failed to send request to UniFi Network API")?;

		if !response.status().is_success() {
			let status = response.status();
			let error_text = response.text().await.unwrap_or_default();

			// Try to parse as API error
			if let Ok(api_error) = serde_json::from_str::<ApiError>(&error_text) {
				anyhow::bail!(
					"UniFi Network API error ({}): {} - {}",
					status,
					api_error.name,
					api_error.error
				);
			}

			anyhow::bail!("UniFi Network API error ({}): {}", status, error_text);
		}

		response
			.json::<SitesResponse>()
			.await
			.context("Failed to parse sites response")
	}

	async fn get_devices(&self, site_id: &str) -> Result<DevicesResponse> {
		let url = format!("{}/sites/{}/devices?limit=200", self.base_url, site_id);

		let response = self
			.client
			.get(&url)
			.header("X-API-KEY", &self.api_token)
			.header("Accept", "application/json")
			.send()
			.await
			.context("Failed to send request to UniFi Network API")?;

		if !response.status().is_success() {
			let status = response.status();
			let error_text = response.text().await.unwrap_or_default();
			anyhow::bail!("Failed to get devices ({}): {}", status, error_text);
		}

		response
			.json::<DevicesResponse>()
			.await
			.context("Failed to parse devices response")
	}

	async fn get_device_statistics(
		&self,
		site_id: &str,
		device_id: &str,
	) -> Result<DeviceStatistics> {
		let url = format!(
			"{}/sites/{}/devices/{}/statistics/latest",
			self.base_url, site_id, device_id
		);

		let response = self
			.client
			.get(&url)
			.header("X-API-KEY", &self.api_token)
			.header("Accept", "application/json")
			.send()
			.await
			.context("Failed to send request to UniFi Network API")?;

		if !response.status().is_success() {
			let status = response.status();
			let error_text = response.text().await.unwrap_or_default();
			anyhow::bail!(
				"Failed to get device statistics ({}): {}",
				status,
				error_text
			);
		}

		response
			.json::<DeviceStatistics>()
			.await
			.context("Failed to parse device statistics")
	}
}
