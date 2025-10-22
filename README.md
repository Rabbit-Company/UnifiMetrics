# 🐇 UnifiMetrics

UnifiMetrics is a Rust-based monitoring tool that collects metrics from UniFi Network and Protect applications. It provides real-time monitoring of your UniFi ecosystem and exposes metrics in OpenMetrics format for integration with external monitoring systems like Prometheus.

## 🚀 Features

- 🌐 **UniFi Integration** - Connects to both UniFi Network and Protect applications
- 📊 **Device Monitoring** - Tracks access points, switches, gateways, and security gateways
- 📹 **Protect Sensor Tracking** - Monitors cameras, doorbells, and environmental sensors
- ⚙️ **Flexible Configuration** - Simple, TOML-based configuration file
- 🧾 **Comprehensive Logging** - Adjustable log levels for detailed diagnostics
- 📊 **Metrics Endpoint** - OpenMetrics format for Prometheus and similar tools

## ⚙️ Configuration

Before running UnifiMetrics, create a `config.toml` file with your settings:

```toml
# ============================================================
# UniFi Metrics Configuration
# ============================================================

[unifi]
# IP address or hostname of the UniFi Controller
ip = "10.0.0.1"

# API token used for authenticating requests to the UniFi Controller
api_token = ""

# Time interval (in seconds) between each data polling cycle
poll_interval = 30


[monitoring]
# Enable or disable collection of UniFi Network device metrics (e.g., APs, switches, gateways)
network_devices = true

# Enable or disable collection of UniFi Protect sensor metrics (e.g., cameras, doorbells)
protect_sensors = true


[server]
# IP address to bind the metrics server to
# Use "0.0.0.0" to listen on all available network interfaces
bind_address = "0.0.0.0"

# Port on which the metrics HTTP endpoint will be exposed
port = 8089

# Optional security token for authenticating access to the metrics endpoint
# If not specified, the endpoint will be publicly accessible
bearer_token = "secure-monitoring-token-123"


[logging]
# Absolute path to the log file where output will be written
# If not specified, logs won't get written to a file
log_file = "/var/log/unifimetrics.log"

# Logging verbosity level — possible values:
#   trace | debug | info | warn | error
# Recommended: "info" for production, "debug" for troubleshooting
log_level = "info"
```

## 🧩 Installation

```bash
# Download the binary
wget https://github.com/Rabbit-Company/UnifiMetrics/releases/latest/download/unifimetrics-$(uname -m)-gnu
# Set file permissions
sudo chmod 755 unifimetrics-$(uname -m)-gnu
# Place the binary to `/usr/local/bin`
sudo mv unifimetrics-$(uname -m)-gnu /usr/local/bin/unifimetrics
# Start the monitor and don't forget to change the path to your config.toml file
unifimetrics /etc/unifimetrics/config.toml
```

## 🧠 Daemonizing (using systemd)

Running UnifiMetrics in the background is a simple task, just make sure that it runs without errors before doing this. Place the contents below in a file called `unifimetrics.service` in the `/etc/systemd/system/` directory.

```service
[Unit]
Description=UnifiMetrics
After=network.target

[Service]
Type=simple
User=root
ExecStart=unifimetrics /etc/unifimetrics/config.toml
TimeoutStartSec=0
TimeoutStopSec=2
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
```

Then, run the commands below to reload systemd and start UnifiMetrics.

```bash
systemctl enable --now unifimetrics
```

## 🔄 Upgrade

```bash
# Stop service
systemctl stop unifimetrics

# Download Pulse Monitor
wget https://github.com/Rabbit-Company/UnifiMetrics/releases/latest/download/unifimetrics-$(uname -m)-gnu
sudo chmod 755 unifimetrics-$(uname -m)-gnu
sudo mv unifimetrics-$(uname -m)-gnu /usr/local/bin/unifimetrics

# Start service
systemctl start unifimetrics
```
