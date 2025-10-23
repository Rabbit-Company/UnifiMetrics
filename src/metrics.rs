use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use std::collections::HashMap;
use std::fmt::Write;
use std::sync::RwLock;

use crate::AppState;

// Metric storage
pub struct MetricsStore {
	device_metrics: RwLock<HashMap<String, DeviceMetrics>>,
	sensor_metrics: RwLock<HashMap<String, SensorMetrics>>,
	poll_metrics: RwLock<HashMap<String, PollMetrics>>,
}

#[derive(Clone, Debug)]
pub struct DeviceMetrics {
	pub site_id: String,
	pub site_name: String,
	pub device_id: String,
	pub device_name: String,
	pub device_model: String,
	pub ip_address: String,
	pub cpu_usage: Option<f64>,
	pub memory_usage: Option<f64>,
	pub uplink_tx_rate: Option<f64>,
	pub uplink_rx_rate: Option<f64>,
	pub state: i32,
}

#[derive(Clone, Debug)]
pub struct SensorMetrics {
	pub sensor_id: String,
	pub sensor_name: String,
	pub mount_type: String,
	pub temperature: Option<f64>,
	pub humidity: Option<f64>,
	pub light: Option<f64>,
	pub battery: Option<f64>,
	pub state: i32,
	pub motion_detected: Option<i32>,
	pub is_opened: Option<i32>,
}

#[derive(Clone, Debug)]
pub struct PollMetrics {
	pub success: i32,
	pub duration: f64,
	//pub timestamp: Instant,
}

// Global metrics store
static METRICS: once_cell::sync::Lazy<MetricsStore> = once_cell::sync::Lazy::new(|| MetricsStore {
	device_metrics: RwLock::new(HashMap::new()),
	sensor_metrics: RwLock::new(HashMap::new()),
	poll_metrics: RwLock::new(HashMap::new()),
});

// Device metrics update functions
pub fn update_device_metrics(
	site_id: &str,
	site_name: &str,
	device_id: &str,
	device_name: &str,
	device_model: &str,
	ip_address: &str,
	cpu_usage: Option<f64>,
	memory_usage: Option<f64>,
	uplink_tx_rate: Option<f64>,
	uplink_rx_rate: Option<f64>,
	state: i32,
) {
	let key = format!("{}_{}", site_id, device_id);
	let metrics = DeviceMetrics {
		site_id: site_id.to_string(),
		site_name: site_name.to_string(),
		device_id: device_id.to_string(),
		device_name: device_name.to_string(),
		device_model: device_model.to_string(),
		ip_address: ip_address.to_string(),
		cpu_usage,
		memory_usage,
		uplink_tx_rate,
		uplink_rx_rate,
		state,
	};

	let mut store = METRICS.device_metrics.write().unwrap();
	store.insert(key, metrics);
}

// Sensor metrics update functions
pub fn update_sensor_metrics(
	sensor_id: &str,
	sensor_name: &str,
	mount_type: &str,
	temperature: Option<f64>,
	humidity: Option<f64>,
	light: Option<f64>,
	battery: Option<f64>,
	state: i32,
	motion_detected: Option<i32>,
	is_opened: Option<i32>,
) {
	let metrics = SensorMetrics {
		sensor_id: sensor_id.to_string(),
		sensor_name: sensor_name.to_string(),
		mount_type: mount_type.to_string(),
		temperature,
		humidity,
		light,
		battery,
		state,
		motion_detected,
		is_opened,
	};

	let mut store = METRICS.sensor_metrics.write().unwrap();
	store.insert(sensor_id.to_string(), metrics);
}

// Poll metrics update functions
pub fn update_poll_metrics(poll_type: &str, success: bool, duration: f64) {
	let metrics = PollMetrics {
		success: if success { 1 } else { 0 },
		duration,
	};

	let mut store = METRICS.poll_metrics.write().unwrap();
	store.insert(poll_type.to_string(), metrics);
}

// Generate OpenMetrics format output
fn generate_metrics_output() -> String {
	let mut output = String::new();

	// Add device metrics
	let devices = METRICS.device_metrics.read().unwrap();

	if !devices.is_empty() {
		// CPU Usage
		writeln!(
			output,
			"# HELP unifi_device_cpu_usage_ratio CPU usage of devices as a normalized ratio between 0.0 and 1.0."
		)
		.unwrap();
		writeln!(output, "# TYPE unifi_device_cpu_usage_ratio gauge").unwrap();
		writeln!(output, "# UNIT unifi_device_cpu_usage_ratio ratio").unwrap();
		for device in devices.values() {
			if let Some(cpu) = device.cpu_usage {
				writeln!(
					output,
					r#"unifi_device_cpu_usage_ratio{{site_id="{}",site_name="{}",device_id="{}",device_name="{}",device_model="{}",ip_address="{}"}} {}"#,
					device.site_id, device.site_name, device.device_id, device.device_name, device.device_model, device.ip_address, cpu / 100.0
				).unwrap();
			}
		}

		// Memory Usage
		writeln!(
			output,
			"# HELP unifi_device_memory_usage_ratio Memory usage of devices as a normalized ratio between 0.0 and 1.0."
		)
		.unwrap();
		writeln!(output, "# TYPE unifi_device_memory_usage_ratio gauge").unwrap();
		writeln!(output, "# UNIT unifi_device_memory_usage_ratio ratio").unwrap();
		for device in devices.values() {
			if let Some(memory) = device.memory_usage {
				writeln!(
					output,
					r#"unifi_device_memory_usage_ratio{{site_id="{}",site_name="{}",device_id="{}",device_name="{}",device_model="{}",ip_address="{}"}} {}"#,
					device.site_id, device.site_name, device.device_id, device.device_name, device.device_model, device.ip_address, memory / 100.0
				).unwrap();
			}
		}

		// Uplink TX Rate
		writeln!(
			output,
			"# HELP unifi_device_upload_speed_bits_per_second Upload speed in bits/sec"
		)
		.unwrap();
		writeln!(
			output,
			"# TYPE unifi_device_upload_speed_bits_per_second gauge"
		)
		.unwrap();
		writeln!(
			output,
			"# UNIT unifi_device_upload_speed_bits_per_second bits_per_second"
		)
		.unwrap();
		for device in devices.values() {
			if let Some(tx_rate) = device.uplink_tx_rate {
				writeln!(
					output,
					r#"unifi_device_upload_speed_bits_per_second{{site_id="{}",site_name="{}",device_id="{}",device_name="{}",device_model="{}",ip_address="{}"}} {}"#,
					device.site_id, device.site_name, device.device_id, device.device_name, device.device_model, device.ip_address, tx_rate
				).unwrap();
			}
		}

		// Uplink RX Rate
		writeln!(
			output,
			"# HELP unifi_device_download_speed_bits_per_second Download speed in bits/sec"
		)
		.unwrap();
		writeln!(
			output,
			"# TYPE unifi_device_download_speed_bits_per_second gauge"
		)
		.unwrap();
		writeln!(
			output,
			"# UNIT unifi_device_download_speed_bits_per_second bits_per_second"
		)
		.unwrap();
		for device in devices.values() {
			if let Some(rx_rate) = device.uplink_rx_rate {
				writeln!(
					output,
					r#"unifi_device_download_speed_bits_per_second{{site_id="{}",site_name="{}",device_id="{}",device_name="{}",device_model="{}",ip_address="{}"}} {}"#,
					device.site_id, device.site_name, device.device_id, device.device_name, device.device_model, device.ip_address, rx_rate
				).unwrap();
			}
		}

		// Device State
		writeln!(
			output,
			"# HELP unifi_device_state Device state (1 = online, 0 = offline)"
		)
		.unwrap();
		writeln!(output, "# TYPE unifi_device_state gauge").unwrap();
		for device in devices.values() {
			writeln!(
				output,
				r#"unifi_device_state{{site_id="{}",site_name="{}",device_id="{}",device_name="{}",device_model="{}",ip_address="{}"}} {}"#,
				device.site_id, device.site_name, device.device_id, device.device_name, device.device_model, device.ip_address, device.state
			).unwrap();
		}
	}

	// Add sensor metrics
	let sensors = METRICS.sensor_metrics.read().unwrap();

	if !sensors.is_empty() {
		// Temperature
		writeln!(
			output,
			"# HELP unifi_sensor_temperature_celsius Temperature reading from sensor in Celsius"
		)
		.unwrap();
		writeln!(output, "# TYPE unifi_sensor_temperature_celsius gauge").unwrap();
		writeln!(output, "# UNIT unifi_sensor_temperature_celsius celsius").unwrap();
		for sensor in sensors.values() {
			if let Some(temp) = sensor.temperature {
				writeln!(
					output,
					r#"unifi_sensor_temperature_celsius{{sensor_id="{}",sensor_name="{}",mount_type="{}"}} {}"#,
					sensor.sensor_id, sensor.sensor_name, sensor.mount_type, temp
				)
				.unwrap();
			}
		}

		// Humidity
		writeln!(
			output,
			"# HELP unifi_sensor_humidity_ratio Current relative humidity measured by the sensor as a normalized ratio between 0.0 and 1.0."
		)
		.unwrap();
		writeln!(output, "# TYPE unifi_sensor_humidity_ratio gauge").unwrap();
		writeln!(output, "# UNIT unifi_sensor_humidity_ratio ratio").unwrap();
		for sensor in sensors.values() {
			if let Some(humidity) = sensor.humidity {
				writeln!(
					output,
					r#"unifi_sensor_humidity_ratio{{sensor_id="{}",sensor_name="{}",mount_type="{}"}} {}"#,
					sensor.sensor_id,
					sensor.sensor_name,
					sensor.mount_type,
					humidity / 100.0
				)
				.unwrap();
			}
		}

		// Light
		writeln!(
			output,
			"# HELP unifi_sensor_light_candela_per_square_meter Current light level measured by the sensor in candela per square meter."
		)
		.unwrap();
		writeln!(
			output,
			"# TYPE unifi_sensor_light_candela_per_square_meter gauge"
		)
		.unwrap();
		writeln!(
			output,
			"# UNIT unifi_sensor_light_candela_per_square_meter candela_per_square_meter"
		)
		.unwrap();
		for sensor in sensors.values() {
			if let Some(light) = sensor.light {
				writeln!(
					output,
					r#"unifi_sensor_light_candela_per_square_meter{{sensor_id="{}",sensor_name="{}",mount_type="{}"}} {}"#,
					sensor.sensor_id, sensor.sensor_name, sensor.mount_type, light
				)
				.unwrap();
			}
		}

		// Battery
		writeln!(
			output,
			"# HELP unifi_sensor_battery_ratio Battery level of the sensor as a normalized ratio between 0.0 and 1.0."
		)
		.unwrap();
		writeln!(output, "# TYPE unifi_sensor_battery_ratio gauge").unwrap();
		writeln!(output, "# UNIT unifi_sensor_battery_ratio ratio").unwrap();
		for sensor in sensors.values() {
			if let Some(battery) = sensor.battery {
				writeln!(
					output,
					r#"unifi_sensor_battery_ratio{{sensor_id="{}",sensor_name="{}",mount_type="{}"}} {}"#,
					sensor.sensor_id,
					sensor.sensor_name,
					sensor.mount_type,
					battery / 100.0
				)
				.unwrap();
			}
		}

		// Sensor State
		writeln!(
			output,
			"# HELP unifi_sensor_state Sensor connection state (1 = connected, 0 = disconnected)"
		)
		.unwrap();
		writeln!(output, "# TYPE unifi_sensor_state gauge").unwrap();
		for sensor in sensors.values() {
			writeln!(
				output,
				r#"unifi_sensor_state{{sensor_id="{}",sensor_name="{}",mount_type="{}"}} {}"#,
				sensor.sensor_id, sensor.sensor_name, sensor.mount_type, sensor.state
			)
			.unwrap();
		}

		// Motion Detection
		writeln!(
			output,
			"# HELP unifi_sensor_motion_detected Motion detection status (1 = detected, 0 = not detected)"
		)
		.unwrap();
		writeln!(output, "# TYPE unifi_sensor_motion_detected gauge").unwrap();
		for sensor in sensors.values() {
			if let Some(motion) = sensor.motion_detected {
				writeln!(
					output,
					r#"unifi_sensor_motion_detected{{sensor_id="{}",sensor_name="{}",mount_type="{}"}} {}"#,
					sensor.sensor_id, sensor.sensor_name, sensor.mount_type, motion
				)
				.unwrap();
			}
		}

		// Door/Window Sensor
		writeln!(
			output,
			"# HELP unifi_sensor_opened Door/window sensor status (1 = opened, 0 = closed)"
		)
		.unwrap();
		writeln!(output, "# TYPE unifi_sensor_opened gauge").unwrap();
		for sensor in sensors.values() {
			if let Some(opened) = sensor.is_opened {
				writeln!(
					output,
					r#"unifi_sensor_opened{{sensor_id="{}",sensor_name="{}",mount_type="{}"}} {}"#,
					sensor.sensor_id, sensor.sensor_name, sensor.mount_type, opened
				)
				.unwrap();
			}
		}
	}

	// Add poll metrics
	let polls = METRICS.poll_metrics.read().unwrap();

	if !polls.is_empty() {
		// Poll Success
		writeln!(
			output,
			"# HELP unifi_poll_success Whether the last poll was successful (1 = success, 0 = failure)"
		)
		.unwrap();
		writeln!(output, "# TYPE unifi_poll_success gauge").unwrap();
		for (poll_type, metrics) in polls.iter() {
			writeln!(
				output,
				r#"unifi_poll_success{{type="{}"}} {}"#,
				poll_type, metrics.success
			)
			.unwrap();
		}

		// Poll Duration
		writeln!(
			output,
			"# HELP unifi_poll_duration_seconds Duration of the last poll in seconds"
		)
		.unwrap();
		writeln!(output, "# TYPE unifi_poll_duration_seconds gauge").unwrap();
		writeln!(output, "# UNIT unifi_poll_duration_seconds seconds").unwrap();
		for (poll_type, metrics) in polls.iter() {
			writeln!(
				output,
				r#"unifi_poll_duration_seconds{{type="{}"}} {}"#,
				poll_type, metrics.duration
			)
			.unwrap();
		}
	}

	// Add EOF marker for OpenMetrics
	writeln!(output, "# EOF").unwrap();

	output
}

pub async fn metrics_handler(headers: HeaderMap, State(state): State<AppState>) -> Response {
	if let Some(ref required_token) = state.bearer_token {
		let auth_header = headers.get("authorization").and_then(|h| h.to_str().ok());

		match auth_header {
			Some(header) if header == format!("Bearer {}", required_token) => {
				// Authorized
			}
			_ => {
				return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
			}
		}
	}

	let metrics_output = generate_metrics_output();

	Response::builder()
		.status(StatusCode::OK)
		.header(
			header::CONTENT_TYPE,
			"application/openmetrics-text; version=1.0.0; charset=utf-8",
		)
		.body(metrics_output.into())
		.unwrap()
}

// Re-export once_cell for use in the module
pub(crate) use once_cell;
