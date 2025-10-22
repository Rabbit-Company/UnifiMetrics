use std::collections::HashMap;
use std::sync::RwLock;

use super::models::{CachedDevice, CachedSite, Device, Site};

pub struct UnifiCache {
	sites: RwLock<HashMap<String, CachedSite>>,
}

impl UnifiCache {
	pub fn new() -> Self {
		Self {
			sites: RwLock::new(HashMap::new()),
		}
	}

	pub fn update_sites(&self, sites: Vec<Site>) {
		let mut cache = self.sites.write().unwrap();
		for site in sites {
			cache.entry(site.id.clone()).or_insert_with(|| CachedSite {
				id: site.id.clone(),
				name: site.name.clone(),
				devices: HashMap::new(),
			});
		}
	}

	pub fn update_devices(&self, site_id: &str, devices: Vec<Device>) {
		let mut cache = self.sites.write().unwrap();
		if let Some(site) = cache.get_mut(site_id) {
			for device in devices {
				site.devices.insert(
					device.id.clone(),
					CachedDevice {
						name: device.name,
						model: device.model,
						ip_address: device.ip_address,
						state: device.state,
					},
				);
			}
		}
	}

	pub fn get_sites(&self) -> Vec<CachedSite> {
		let cache = self.sites.read().unwrap();
		cache.values().cloned().collect()
	}
}
