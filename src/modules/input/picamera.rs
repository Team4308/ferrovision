use crate::modules::InputModule;

use opencv::core::*;
use opencv::prelude::*;
use opencv::videoio::VideoCapture;

use toml::Value;

pub struct PiCameraInput {
	cap: VideoCapture,
}

impl InputModule for PiCameraInput {
	fn new(settings: Value) -> Self {
		let mut cap = VideoCapture::default().unwrap();
		cap.open(0).unwrap();
		cap.set(4, settings["input.picamera"]["height"].as_integer().unwrap() as f64).unwrap();
		cap.set(3, settings["input.picamera"]["width"].as_integer().unwrap() as f64).unwrap();
		cap.set(5, settings["input.picamera"]["fps"].as_integer().unwrap() as f64).unwrap();
		Self {
			cap: cap
		}	
	}
	
	fn run(&mut self) -> Mat {
		let mut frame = Mat::default().unwrap();
		self.cap.read(&mut frame).unwrap();
		frame
	}
}
