pub mod input;
pub mod threshold;
pub mod filter;
pub mod output;

use opencv::types::*;
use opencv::core::*;

use std::any::Any;

pub struct TrackingData {
	pub cnt: VectorOfPoint,
	pub hull: VectorOfPoint,
	pub rect: RotatedRect,
	pub bounding: Rect,
}

pub struct OutputData {
	pub raw_center: [f64; 2],
	pub normal_coord: [f64; 2],
	pub angle: f64,
}

pub trait InputModule {
	fn run(&mut self) -> Mat;
	fn as_any(&mut self) -> &dyn Any;
}

pub trait ThresholdModule {
	fn run(&mut self, frame: &Mat) -> Vec<TrackingData>;
	fn as_any(&mut self) -> &dyn Any;
}

pub trait FilterModule {
	fn run(&mut self, object: &TrackingData) -> bool;
	fn as_any(&mut self) -> &dyn Any;
}


pub trait OutputModule {
	fn run(&mut self, data: OutputData);
	fn as_any(&mut self) -> &dyn Any;
}