use nt::*;

use crate::modules::OutputModule;
use crate::modules::{OutputData};

use toml::Value;

use std::any::Any;

pub struct NetworkTable {
	client: NetworkTables<Client>,

	raw_center_id: u16,
	normal_coord_id: u16,
	angle_id: u16,
}

impl OutputModule for NetworkTable {
	fn run(&mut self, object: OutputData) {
		self.client.update_entry(self.raw_center_id, EntryValue::DoubleArray(object.raw_center.to_vec()));
		self.client.update_entry(self.normal_coord_id, EntryValue::DoubleArray(object.normal_coord.to_vec()));
		self.client.update_entry(self.angle_id, EntryValue::Double(object.angle));
	}

	fn as_any(&mut self) -> &dyn Any {
		self
	}
}

impl NetworkTable {
	pub async fn new(settings: &Value) -> Self {
		let client = NetworkTables::connect(settings["output"]["ip"].as_str().unwrap(), "ferrovision").await.unwrap();

		let raw_center_id = client.create_entry(EntryData::new("ff_raw_center".to_string(), 0, EntryValue::DoubleArray(vec![0., 0.]))).await.unwrap();
		let normal_coord_id = client.create_entry(EntryData::new("ff_normal_coord".to_string(), 0, EntryValue::DoubleArray(vec![0., 0.]))).await.unwrap();
		let angle_id = client.create_entry(EntryData::new("ff_angle".to_string(), 0, EntryValue::Double(0.))).await.unwrap();
		
		Self {
			client: client,

			raw_center_id: raw_center_id,
			normal_coord_id: normal_coord_id,
			angle_id: angle_id,
		}
	}
}