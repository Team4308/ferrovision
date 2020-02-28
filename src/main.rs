use opencv::prelude::*;
use opencv::imgcodecs::*;
use opencv::types::*;
use opencv::imgproc;
use opencv::core::*;

use toml::{self, Value};

use itertools::Itertools;

use std::io::{self, Write};
use std::io::prelude::*;
use std::cmp::Ordering;
use std::fs::File;

mod modules;
use crate::modules::OutputData;

use crate::modules::InputModule;
use crate::modules::input::PiCameraInput;

use crate::modules::ThresholdModule;
use crate::modules::threshold::SimpleColor;
use crate::modules::threshold::ComplexColor;

use crate::modules::FilterModule;
use crate::modules::filter::ContourArea;
use crate::modules::filter::PercentFilled;

use crate::modules::OutputModule;
use crate::modules::output::NetworkTable;

// Main
#[tokio::main]
async fn main() {
	//Load Vision Settings
	let mut vset_file = File::open("vset.toml").unwrap();
	let mut vset_content = String::new();
	vset_file.read_to_string(&mut vset_content).unwrap();
	let vset: Value = toml::from_str(&vset_content).unwrap();

	let vset_height = vset["input"]["height"].as_integer().unwrap();
	let vset_width = vset["input"]["height"].as_integer().unwrap();
	let num_tracked_contours = vset["tracking"]["num_tracked_contours"].as_integer().unwrap();
	
	//Input Module
	let input_module_toml = vset["pipeline"]["input"].as_str().unwrap();
	let mut input: Box<dyn InputModule> = match input_module_toml {
		"picamera" => Box::new(PiCameraInput::new(&vset)),
		_ => panic!("Input Module Unknown! | {}", input_module_toml),
	};

	// Threshold Module
	let theshold_module_toml = vset["pipeline"]["threshold"].as_str().unwrap();
	let mut threshold: Box<dyn ThresholdModule> = match theshold_module_toml {
		"simplecolor" => Box::new(SimpleColor::new(&vset)),
		"complexcolor" => Box::new(ComplexColor::new(&vset)),
		_ => panic!("Theshold Module Unknown! | {}", theshold_module_toml),
	};

	// Filter Modules
	let filter_modules_toml = vset["pipeline"]["filter"].as_array().unwrap().iter().map(|a| a.as_str().unwrap()).collect::<Vec<&str>>();
	let mut filter_modules = Vec::<Box<dyn FilterModule>>::new();
	for filter_toml in filter_modules_toml {
		let filter: Box<dyn FilterModule> = match filter_toml {
			"contourarea" => Box::new(ContourArea::new(&vset)),
			"percentfilled" => Box::new(PercentFilled::new(&vset)),
			_ => panic!("Filter Module(s) Unknown! | {}", filter_toml),
		};
		filter_modules.push(filter);
	}

	// Output Module
	let output_module_toml = vset["pipeline"]["output"].as_str().unwrap();
	let mut output: Box<dyn OutputModule> = match output_module_toml {
		"nt" => Box::new(NetworkTable::new(&vset).await),
		_ => panic!("Output Module Unknown! | {}", output_module_toml),
	};

	//Init Params For Encoding
	let params: VectorOfint = VectorOfint::new();
	
	let mut prev_tracked_center: [f64; 2] = [0., 0.];
	
	//Main Vision Loop
	loop {
		// FPS
		let e1 = get_tick_count().unwrap();

		let mut frame: Mat = input.run();

		//let se1 = imgproc::get_structuring_element(imgproc::MORPH_RECT, Size::new(4, 4), Point::new(-1, -1)).unwrap();
		//let se2 = imgproc::get_structuring_element(imgproc::MORPH_RECT, Size::new(2, 2), Point::new(-1, -1)).unwrap();

		let mut tracked_objects = threshold.run(&frame);

		for module in filter_modules.iter_mut() {
			tracked_objects.retain(|obj| module.run(obj));
		}

		let tracked_objects_sorted = tracked_objects.iter().sorted_by(|a, b| contour_area_simple(&a.cnt).partial_cmp(&contour_area_simple(&b.cnt)).unwrap_or(Ordering::Equal));

		let mut tracked_center: [f64; 2] = [0., 0.];
		let mut object_count = 0;
		for obj in tracked_objects_sorted.rev() {
			let mut cnt_draw = opencv::types::VectorOfVectorOfPoint::new();
			cnt_draw.push(opencv::types::VectorOfPoint::from_iter(obj.cnt.iter()));
			imgproc::draw_contours(&mut frame, &cnt_draw, -1, Scalar::new(255., 0., 0., 0.), 2, LINE_8, &no_array().unwrap(), i32::max_value(), Point::new(0, 0)).unwrap();	

			imgproc::rectangle(&mut frame, obj.bounding.clone(), Scalar::new(0., 255., 255., 0.), 2, LINE_8, 0).unwrap();

			tracked_center[0] += obj.bounding.x as f64;
			tracked_center[1] += obj.bounding.y as f64;

			object_count += 1;
			if object_count >= num_tracked_contours {
				break
			}
		}

		tracked_center[0] = tracked_center[0] / num_tracked_contours as f64;
		tracked_center[1] = tracked_center[1] / num_tracked_contours as f64;

		if prev_tracked_center != tracked_center && tracked_center != [0., 0.] {
			let nx = (1. / (vset_width as f64 / 2.)) * (tracked_center[0] - ((vset_width as f64 / 2.) - 0.5));
			let ny = (1. / (vset_height as f64 / 2.)) * (((vset_height as f64 / 2.) - 0.5) - tracked_center[1]);

			let vpw = 2. * (62.2f64 / 2.).tan();
			let vph = 2. * (48.8f64 / 2.).tan();

			let x = vpw / 2. * nx;
			let y = vph / 2. * ny;

			let ax = 1.0f64.atan2(x);

			let output_data = OutputData {
				raw_center: tracked_center,
				normal_coord: [nx, ny],
				angle: ax,
			};

			output.run(output_data);

			prev_tracked_center = tracked_center;
		}

		// FPS
		let e2 = get_tick_count().unwrap();
		let fps = 1. / ((e2 as f64 - e1 as f64) / get_tick_frequency().unwrap());

		imgproc::put_text(&mut frame, &format!("{}", fps as usize)[..], Point::new(10, (vset_height / 8) as i32), FONT_HERSHEY_DUPLEX, 0.5, Scalar::new(255., 255., 0., 0.), 2, LINE_8, false).unwrap();
		
		// Sending Image
		let mut sized = Mat::default().unwrap();
		imgproc::resize(&frame, &mut sized, Size::default(), 0.5, 0.5, imgproc::INTER_AREA).unwrap();
		let jpeg = to_jpeg(&mut frame, &params);
		io::stdout().write_all(&jpeg).unwrap();
		io::stdout().flush().unwrap();
	}
}

fn to_jpeg(mut frame: & Mat, params: &VectorOfint) -> std::vec::Vec<u8> {
	let mut jpeg = Vector::new();
	imencode(".jpg", &mut frame, &mut jpeg, &params).unwrap();
	jpeg.to_vec()
}

fn contour_area_simple(contour: &dyn ToInputArray) -> f64 {
	imgproc::contour_area(contour, false).unwrap()
}