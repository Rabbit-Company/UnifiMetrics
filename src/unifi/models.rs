use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Common error response
#[derive(Debug, Deserialize, Serialize)]
pub struct ApiError {
	pub error: String,
	pub name: String,
	pub cause: Option<ErrorCause>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorCause {
	pub error: String,
	pub name: String,
}

// Network API models
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SitesResponse {
	pub offset: Option<u32>,
	pub limit: Option<u32>,
	pub count: Option<u32>,
	#[serde(rename = "totalCount")]
	pub total_count: Option<u32>,
	pub data: Vec<Site>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Site {
	pub id: String,
	#[serde(rename = "internalReference")]
	pub internal_reference: Option<String>,
	pub name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DevicesResponse {
	pub offset: Option<u32>,
	pub limit: Option<u32>,
	pub count: Option<u32>,
	#[serde(rename = "totalCount")]
	pub total_count: Option<u32>,
	pub data: Vec<Device>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Device {
	pub id: String,
	pub name: String,
	pub model: String,
	#[serde(rename = "macAddress")]
	pub mac_address: String,
	#[serde(rename = "ipAddress")]
	pub ip_address: Option<String>,
	pub state: String,
	pub features: Option<Vec<String>>,
	pub interfaces: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeviceStatistics {
	#[serde(rename = "uptimeSec")]
	pub uptime_sec: Option<i64>,
	#[serde(rename = "lastHeartbeatAt")]
	pub last_heartbeat_at: Option<String>,
	#[serde(rename = "nextHeartbeatAt")]
	pub next_heartbeat_at: Option<String>,
	#[serde(rename = "loadAverage1Min")]
	pub load_average_1min: Option<f64>,
	#[serde(rename = "loadAverage5Min")]
	pub load_average_5min: Option<f64>,
	#[serde(rename = "loadAverage15Min")]
	pub load_average_15min: Option<f64>,
	#[serde(rename = "cpuUtilizationPct")]
	pub cpu_utilization_pct: Option<f64>,
	#[serde(rename = "memoryUtilizationPct")]
	pub memory_utilization_pct: Option<f64>,
	pub uplink: Option<UplinkStats>,
	pub interfaces: Option<InterfaceStats>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UplinkStats {
	#[serde(rename = "txRateBps")]
	pub tx_rate_bps: Option<f64>,
	#[serde(rename = "rxRateBps")]
	pub rx_rate_bps: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InterfaceStats {
	pub radios: Option<Vec<RadioStats>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RadioStats {
	#[serde(rename = "frequencyGHz")]
	pub frequency_ghz: Option<f64>,
	#[serde(rename = "txRetriesPct")]
	pub tx_retries_pct: Option<f64>,
}

// Protect API models
#[derive(Debug, Deserialize, Serialize)]
pub struct Sensor {
	pub id: String,
	#[serde(rename = "modelKey")]
	pub model_key: String,
	pub state: String,
	pub name: String,
	#[serde(rename = "mountType")]
	pub mount_type: Option<String>,
	#[serde(rename = "batteryStatus")]
	pub battery_status: Option<BatteryStatus>,
	pub stats: Option<SensorStats>,
	#[serde(rename = "lightSettings")]
	pub light_settings: Option<SensorSettings>,
	#[serde(rename = "humiditySettings")]
	pub humidity_settings: Option<SensorSettings>,
	#[serde(rename = "temperatureSettings")]
	pub temperature_settings: Option<SensorSettings>,
	#[serde(rename = "isOpened")]
	pub is_opened: Option<bool>,
	#[serde(rename = "openStatusChangedAt")]
	pub open_status_changed_at: Option<i64>,
	#[serde(rename = "isMotionDetected")]
	pub is_motion_detected: Option<bool>,
	#[serde(rename = "motionDetectedAt")]
	pub motion_detected_at: Option<i64>,
	#[serde(rename = "motionSettings")]
	pub motion_settings: Option<MotionSettings>,
	#[serde(rename = "alarmTriggeredAt")]
	pub alarm_triggered_at: Option<i64>,
	#[serde(rename = "alarmSettings")]
	pub alarm_settings: Option<AlarmSettings>,
	#[serde(rename = "leakDetectedAt")]
	pub leak_detected_at: Option<i64>,
	#[serde(rename = "externalLeakDetectedAt")]
	pub external_leak_detected_at: Option<i64>,
	#[serde(rename = "leakSettings")]
	pub leak_settings: Option<LeakSettings>,
	#[serde(rename = "tamperingDetectedAt")]
	pub tampering_detected_at: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BatteryStatus {
	pub percentage: Option<f64>,
	#[serde(rename = "isLow")]
	pub is_low: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SensorStats {
	pub light: Option<SensorValue>,
	pub humidity: Option<SensorValue>,
	pub temperature: Option<SensorValue>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SensorValue {
	pub value: Option<f64>,
	pub status: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SensorSettings {
	#[serde(rename = "isEnabled")]
	pub is_enabled: Option<bool>,
	pub margin: Option<f64>,
	#[serde(rename = "lowThreshold")]
	pub low_threshold: Option<f64>,
	#[serde(rename = "highThreshold")]
	pub high_threshold: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MotionSettings {
	#[serde(rename = "isEnabled")]
	pub is_enabled: Option<bool>,
	pub sensitivity: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AlarmSettings {
	#[serde(rename = "isEnabled")]
	pub is_enabled: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LeakSettings {
	#[serde(rename = "isInternalEnabled")]
	pub is_internal_enabled: Option<bool>,
	#[serde(rename = "isExternalEnabled")]
	pub is_external_enabled: Option<bool>,
}

// Cache structures
#[derive(Debug, Clone)]
pub struct CachedSite {
	pub id: String,
	pub name: String,
	pub devices: HashMap<String, CachedDevice>,
}

#[derive(Debug, Clone)]
pub struct CachedDevice {
	pub name: String,
	pub model: String,
	pub ip_address: Option<String>,
	pub state: String,
}
