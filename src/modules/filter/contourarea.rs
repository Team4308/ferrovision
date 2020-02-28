use crate::modules::FilterModule;
use crate::modules::{TrackingData};

use opencv::imgproc;

use toml::Value;

use std::any::Any;

pub struct ContourArea {
	min: f64,
	max: f64,
	frame_area: i64,
}

impl FilterModule for ContourArea {
	fn run(&mut self, object: &TrackingData) -> bool {
		let percent_area = (imgproc::contour_area(&object.hull, false).unwrap() / self.frame_area as f64) * 100.;
		percent_area > self.min && percent_area < self.max
	}

	fn as_any(&mut self) -> &dyn Any {
		self
	}
}

impl ContourArea {
	pub fn new(settings: &Value) -> Self {
		Self {
			min: settings["filter"]["contourarea"]["min"].as_float().unwrap(),
			max: settings["filter"]["contourarea"]["max"].as_float().unwrap(),
			frame_area: settings["input"]["width"].as_integer().unwrap() * settings["input"]["height"].as_integer().unwrap(),
		}
	}
}