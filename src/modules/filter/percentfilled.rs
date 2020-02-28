use crate::modules::FilterModule;
use crate::modules::{TrackingData};

use opencv::imgproc;

use toml::Value;

use std::any::Any;

pub struct PercentFilled {
	min: f64,
	max: f64,
}

impl FilterModule for PercentFilled {
	fn run(&mut self, object: &TrackingData) -> bool {
		let rect_area = object.rect.size().unwrap().area();
		let cnt_area = imgproc::contour_area(&object.cnt, false).unwrap();
		let percent_area = (cnt_area as f64 / rect_area as f64) * 100.;
		
		percent_area > self.min && percent_area < self.max
	}

	fn as_any(&mut self) -> &dyn Any {
		self
	}
}

impl PercentFilled {
	pub fn new(settings: &Value) -> Self {
		Self {
			min: settings["filter"]["percentfilled"]["min"].as_float().unwrap(),
			max: settings["filter"]["percentfilled"]["max"].as_float().unwrap(),
		}
	}
}