pub mod input;
pub mod threshold;
pub mod filter;
pub mod output;

use opencv::types::*;
use opencv::core::*;

use toml::Value;

pub struct TrackingData {
	pub cnt: VectorOfPoint,
	pub hull: VectorOfPoint,
	pub rect: RotatedRect,
	pub bounding: Rect,
}

pub struct OutputData {
	
}

pub trait InputModule {
	fn new(settings: Value) -> Self;
	fn run(&mut self) -> Mat;
}

pub trait ThresholdModule {
	fn new(settings: Value) -> Self;
	fn run(&mut self) -> TrackingData;
}

pub trait FilterModule {
	fn new(settings: Value) -> Self;
	fn run(&mut self, object: TrackingData) -> bool;
}

pub trait OutputModule {
	fn new(settings: Value) -> Self;
	fn run(&mut self, data: OutputData);
}