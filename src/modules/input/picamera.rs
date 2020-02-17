use crate::modules::InputModule;

use opencv::core::*;
use opencv::prelude::*;
use opencv::videoio::VideoCapture;

use toml::Value;

use std::any::Any;

pub struct PiCameraInput {
	cap: VideoCapture,
}

impl InputModule for PiCameraInput {
	fn run(&mut self) -> Mat {
		let mut frame = Mat::default().unwrap();
		self.cap.read(&mut frame).unwrap();
		frame
	}

	fn as_any(&mut self) -> &dyn Any {
		self
	}
}

impl PiCameraInput {
	pub fn new(settings: &Value) -> Self {
		let mut cap = VideoCapture::default().unwrap();
		cap.open(0).unwrap();
		cap.set(4, settings["input"]["height"].as_integer().unwrap() as f64).unwrap();
		cap.set(3, settings["input"]["width"].as_integer().unwrap() as f64).unwrap();
		cap.set(5, settings["input"]["fps"].as_integer().unwrap() as f64).unwrap();
		Self {
			cap: cap
		}	
	}
}