use anyhow::{Context, Result};
use log::{debug, error, info};
use reqwest::Client;
use std::time::Instant;

use super::models::{ApiError, Sensor};
use crate::metrics::{update_poll_metrics, update_sensor_metrics};

pub struct ProtectClient {
	client: Client,
	base_url: String,
	api_token: String,
}

impl ProtectClient {
	pub fn new(client: Client, ip: String, api_token: String) -> Self {
		Self {
			client,
			base_url: format!("https://{}/proxy/protect/integration/v1", ip),
			api_token,
		}
	}

	pub async fn poll_sensors(&self) -> Result<()> {
		let start = Instant::now();
		let mut success = true;

		match self.get_sensors().await {
			Ok(sensors) => {
				info!("Found {} sensors", sensors.len());

				for sensor in sensors {
					self.update_sensor_metrics_internal(&sensor);
				}
			}
			Err(e) => {
				error!("Failed to fetch sensors: {}", e);
				success = false;
			}
		}

		let duration = start.elapsed().as_secs_f64();
		update_poll_metrics("protect", success, duration);

		Ok(())
	}

	fn update_sensor_metrics_internal(&self, sensor: &Sensor) {
		let mount_type = sensor.mount_type.as_deref().unwrap_or("unknown");

		debug!(
			"Updating metrics for sensor: {} ({})",
			sensor.name, sensor.id
		);

		// Determine sensor state
		let state_value = if sensor.state.to_uppercase() == "CONNECTED" {
			1
		} else {
			0
		};

		// Extract battery percentage
		let battery_pct = sensor.battery_status.as_ref().and_then(|b| b.percentage);

		// Extract sensor stats
		let (temperature, humidity, light) = if let Some(stats) = &sensor.stats {
			let temp = stats.temperature.as_ref().and_then(|t| t.value);
			let humidity = stats.humidity.as_ref().and_then(|h| h.value);
			let light = stats.light.as_ref().and_then(|l| l.value);

			(temp, humidity, light)
		} else {
			(None, None, None)
		};

		// Motion and door/window status
		let motion_detected = sensor.is_motion_detected.map(|b| if b { 1 } else { 0 });
		let is_opened = sensor.is_opened.map(|b| if b { 1 } else { 0 });

		// Update all metrics
		update_sensor_metrics(
			&sensor.id,
			&sensor.name,
			mount_type,
			temperature,
			humidity,
			light,
			battery_pct,
			state_value,
			motion_detected,
			is_opened,
		);
	}

	async fn get_sensors(&self) -> Result<Vec<Sensor>> {
		let url = format!("{}/sensors", self.base_url);

		debug!("Fetching sensors from: {}", url);

		let response = self
			.client
			.get(&url)
			.header("X-API-KEY", &self.api_token)
			.header("Accept", "application/json")
			.send()
			.await
			.context("Failed to send request to UniFi Protect API")?;

		if !response.status().is_success() {
			let status = response.status();
			let error_text = response.text().await.unwrap_or_default();

			// Try to parse as API error
			if let Ok(api_error) = serde_json::from_str::<ApiError>(&error_text) {
				if api_error.name == "API_ERROR" || api_error.name == "UNKNOWN_ERROR" {
					anyhow::bail!(
						"UniFi Protect API authentication failed. Please check your API token. Error: {} - {}",
						api_error.name,
						api_error.error
					);
				}
				anyhow::bail!(
					"UniFi Protect API error ({}): {} - {}",
					status,
					api_error.name,
					api_error.error
				);
			}

			anyhow::bail!("UniFi Protect API error ({}): {}", status, error_text);
		}

		let sensors = response
			.json::<Vec<Sensor>>()
			.await
			.context("Failed to parse sensors response")?;

		debug!("Successfully fetched {} sensors", sensors.len());

		Ok(sensors)
	}
}
