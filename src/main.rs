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
use std::convert::TryInto;
use std::any::Any;

mod modules;
use crate::modules::TrackingData;

use crate::modules::InputModule;
use crate::modules::input::PiCameraInput;

use crate::modules::FilterModule;
use crate::modules::filter::ContourArea;

struct VisionSettings {
	frame_width: i64,
	frame_height: i64,
	frame_size: i64,

	color_l: [u8; 3],
	color_u: [u8; 3],

	num_tracked_contours: i64,

	cnt_area_min: f64,
	cnt_area_max: f64,

	percent_filled_min: f64,
	percent_filled_max: f64,

	zmq_ip: String,
}

// Main
fn main() {
	//Load Vision Settings
	let mut vset_file = File::open("vset.toml").unwrap();
	let mut vset_content = String::new();
	vset_file.read_to_string(&mut vset_content).unwrap();
	let vset: Value = toml::from_str(&vset_content).unwrap();

	let vset_width = vset["input"]["width"].as_integer().unwrap();
	let vset_height = vset["input"]["height"].as_integer().unwrap();
	let vset_frame_size = vset_width * vset_height;

	let color_l: [u8; 3] = vset["threshold"]["colorl"].as_array().unwrap().iter().map(|a| a.as_integer().unwrap() as u8).collect::<Vec<u8>>()[..3].try_into().unwrap();
	let color_u: [u8; 3] = vset["threshold"]["coloru"].as_array().unwrap().iter().map(|a| a.as_integer().unwrap() as u8).collect::<Vec<u8>>()[..3].try_into().unwrap();

	let num_tracked_contours = vset["filter"]["num_tracked_contours"].as_integer().unwrap();

	let cnt_area_min = vset["filter"]["cnt_area_min"].as_float().unwrap();
	let cnt_area_max = vset["filter"]["cnt_area_max"].as_float().unwrap();

	let percent_filled_min = vset["filter"]["percent_filled_min"].as_float().unwrap();
	let percent_filled_max = vset["filter"]["percent_filled_max"].as_float().unwrap();

	let zmq_ip = String::from(vset["output"]["zmq_ip"].as_str().unwrap());

	let vsettings = VisionSettings {
		frame_width: vset_width,
		frame_height: vset_height,
		frame_size: vset_frame_size,

		color_l: color_l,
		color_u: color_u,

		num_tracked_contours: num_tracked_contours,

		cnt_area_min: cnt_area_min,
		cnt_area_max: cnt_area_max,

		percent_filled_min: percent_filled_min,
		percent_filled_max: percent_filled_max,

		zmq_ip: zmq_ip
	};

	// ZeroMQ Init
	
		
	//Input Module
	let mut input = PiCameraInput::new(&vset);

	//Filter Modules
	let mut filter_modules = Vec::<Box<dyn FilterModule>>::new();

	let mut contour_area_filter = ContourArea::new(&vset);
	filter_modules.push(Box::new(contour_area_filter));

	//Init Params For Encoding
	let params: VectorOfint = VectorOfint::new();
	
	
	let mut prev_tracked_center: [f64; 2] = [0., 0.];
	
	//Main Vision Loop
	loop {
		// FPS
		let e1 = get_tick_count().unwrap();

		let mut frame: Mat = input.run();
		
		let mut hsv = Mat::default().unwrap();
		imgproc::cvt_color(&frame, &mut hsv, imgproc::COLOR_BGR2HSV, 0).unwrap();
			
		let mut mask = Mat::default().unwrap();
		in_range(&hsv, &Scalar::new(color_l[0] as f64, color_l[1] as f64, color_l[2] as f64, 0.), &Scalar::new(color_u[0] as f64, color_u[1] as f64, color_u[2] as f64, 0.), &mut mask).unwrap();

		let se1 = imgproc::get_structuring_element(imgproc::MORPH_RECT, Size::new(4, 4), Point::new(-1, -1)).unwrap();
		let se2 = imgproc::get_structuring_element(imgproc::MORPH_RECT, Size::new(2, 2), Point::new(-1, -1)).unwrap();

		let mut tmp_mask = Mat::default().unwrap();
						
		imgproc::morphology_ex(&mask, &mut tmp_mask, imgproc::MORPH_CLOSE, &se1, Point::new(-1, -1), 1, BORDER_CONSTANT, imgproc::morphology_default_border_value().unwrap()).unwrap();
		imgproc::morphology_ex(&tmp_mask, &mut mask, imgproc::MORPH_OPEN, &se2, Point::new(-1, -1), 1, BORDER_CONSTANT, imgproc::morphology_default_border_value().unwrap()).unwrap();

		let mut cnts: opencv::types::VectorOfVectorOfPoint = opencv::types::VectorOfVectorOfPoint::new();
		imgproc::find_contours(&mut mask, &mut cnts, imgproc::RETR_EXTERNAL, imgproc::CHAIN_APPROX_TC89_KCOS, Point::new(0, 0)).unwrap();
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

		imgproc::draw_contours(&mut frame, &cnts, -1, Scalar::new(0., 0., 255., 0.), 2, LINE_8, &no_array().unwrap(), i32::max_value(), Point::new(0, 0)).unwrap();

		for module in filter_modules.iter_mut() {
			tracked_objects.retain(|obj| module.run(obj));
		}

		//tracked_objects.retain(|x| check_object(x, &vsettings));

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
						
			prev_tracked_center = tracked_center;
		}

		// FPS
		let e2 = get_tick_count().unwrap();
		let fps = 1. / ((e2 as f64 - e1 as f64) / get_tick_frequency().unwrap());

		imgproc::put_text(&mut frame, &format!("{}", fps as usize)[..], Point::new(10, (vset_height / 8) as i32), FONT_HERSHEY_DUPLEX, 0.5, Scalar::new(255., 255., 0., 0.), 2, LINE_8, false).unwrap();
					
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

fn check_object(object: &TrackingData, settings: &VisionSettings) -> bool {
	let hull_area = imgproc::contour_area(&object.hull, false).unwrap();
	let percent_area = (hull_area as f64 / settings.frame_size as f64) * 100.;

	if cnt_area_reject(percent_area, settings) {
		let rect_area = object.rect.size().unwrap().area();		
		let cnt_area = imgproc::contour_area(&object.cnt, false).unwrap();
		let percent_filled = (cnt_area as f64 / rect_area as f64) * 100.;

		if percent_filled_reject(percent_filled, settings) {
			return true;
		}
	}

	return false;
}

fn contour_area_simple(contour: &dyn ToInputArray) -> f64 {
	imgproc::contour_area(contour, false).unwrap()
}

fn cnt_area_reject(area: f64, settings: &VisionSettings) -> bool {
	area > settings.cnt_area_min && area < settings.cnt_area_max
}

fn percent_filled_reject(percent_filled: f64, settings: &VisionSettings) -> bool {
	percent_filled > settings.percent_filled_min && percent_filled < settings.percent_filled_max
}
