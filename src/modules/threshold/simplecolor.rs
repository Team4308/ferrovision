use crate::modules::ThresholdModule;
use crate::modules::{TrackingData};

use opencv::prelude::*;
use opencv::imgproc;
use opencv::core::*;

use toml::Value;

use std::any::Any;
use std::convert::TryInto;

pub struct SimpleColor {
	color_l: [u8; 3],
	color_u: [u8; 3],
}

impl ThresholdModule for SimpleColor {
	fn run(&mut self, frame: &Mat) -> Vec<TrackingData> {
		let mut hsv = Mat::default().unwrap();
		imgproc::cvt_color(&frame, &mut hsv, imgproc::COLOR_BGR2HSV, 0).unwrap();
			
		let mut mask = Mat::default().unwrap();
		in_range(&hsv, &Scalar::new(self.color_l[0] as f64, self.color_l[1] as f64, self.color_l[2] as f64, 0.), &Scalar::new(self.color_u[0] as f64, self.color_u[1] as f64, self.color_u[2] as f64, 0.), &mut mask).unwrap();
	
		let mut cnts: opencv::types::VectorOfVectorOfPoint = opencv::types::VectorOfVectorOfPoint::new();
		imgproc::find_contours(&mut mask, &mut cnts, imgproc::RETR_EXTERNAL, imgproc::CHAIN_APPROX_SIMPLE, Point::new(0, 0)).unwrap();
		cnts.shrink_to_fit();

		let mut tracked_objects = Vec::<TrackingData>::new();

		for cnt in cnts.iter() {
			let mut hull = opencv::types::VectorOfPoint::new();
			imgproc::convex_hull(&cnt, &mut hull, false, false).unwrap();
			let rect = imgproc::min_area_rect(&cnt).unwrap();
			let bounding = imgproc::bounding_rect(&cnt).unwrap();
			tracked_objects.push(TrackingData{
				cnt: cnt,
				hull: hull,
				rect: rect,
				bounding: bounding,
			});
		}

		tracked_objects
	}

	fn as_any(&mut self) -> &dyn Any {
		self
	}
}

impl SimpleColor {
	pub fn new(settings: &Value) -> Self {
		Self {
			color_l: settings["threshold"]["colorl"].as_array().unwrap().iter().map(|a| a.as_integer().unwrap() as u8).collect::<Vec<u8>>()[..3].try_into().unwrap(),
			color_u: settings["threshold"]["coloru"].as_array().unwrap().iter().map(|a| a.as_integer().unwrap() as u8).collect::<Vec<u8>>()[..3].try_into().unwrap(),
		}
	}
}