use opencv::prelude::*;
use opencv::videoio::VideoCapture;
use opencv::imgcodecs::*;
use opencv::types::*;
use opencv::imgproc;
use opencv::core::*;

use itertools::Itertools;

use std::io::{self, Write};
use std::io::prelude::*;
use std::thread;
use std::net::TcpListener;
use std::net::TcpStream;
use std::cmp::Ordering;

// Settings
const WIDTH: usize = 640;
const HEIGHT: usize = 480;

const COLOR_L: [u8; 3] = [50, 100, 70];
const COLOR_U: [u8; 3] = [70, 255, 255];

const MAX_CONTOURS: usize = 1;

const CNT_MIN_AREA: f64 = 0.;
const CNT_MAX_AREA: f64 = 100.;

const PERCENT_FILLED_MIN: f64 = 90.;
const PERCENT_FILLED_MAX: f64 = 100.;

// App Start
const INDEX: &str = include_str!("site/index.html");
const FRAME_SIZE: usize = WIDTH * HEIGHT;

fn main() {
	//Vision Thread
	let handle_vision = thread::spawn(|| {
		//Open Camera
		let mut cap = open_camera(0);
		//Init Params For Encoding
		let params: VectorOfint = VectorOfint::new();
		//Init Captured Frame
		let mut frame: Mat = Mat::default().unwrap();
		//Camera Number To Check
		let mut cam_num = 0;

		let mut cnts: opencv::types::VectorOfVectorOfPoint = opencv::types::VectorOfVectorOfPoint::new();
	
		//Main Vision Loop
		loop {
			match cap.read(&mut frame) {
				Ok(cap_read) => match cap_read {
					true => {
						let mut hsv = Mat::default().unwrap();
						imgproc::cvt_color(&frame, &mut hsv, imgproc::COLOR_BGR2HSV, 0).unwrap();
			
						let mut mask = Mat::default().unwrap();
						in_range(&hsv, &Scalar::new(COLOR_L[0] as f64, COLOR_L[1] as f64, COLOR_L[2] as f64, 0.), &Scalar::new(COLOR_U[0] as f64, COLOR_U[1] as f64, COLOR_U[2] as f64, 0.), &mut mask).unwrap();

						let se1 = imgproc::get_structuring_element(imgproc::MORPH_RECT, Size::new(4, 4), Point::new(-1, -1)).unwrap();
						let se2 = imgproc::get_structuring_element(imgproc::MORPH_RECT, Size::new(2, 2), Point::new(-1, -1)).unwrap();

						let mut tmp_mask = Mat::default().unwrap();
						
						imgproc::morphology_ex(&mask, &mut tmp_mask, imgproc::MORPH_CLOSE, &se1, Point::new(-1, -1), 1, BORDER_CONSTANT, imgproc::morphology_default_border_value().unwrap()).unwrap();
						imgproc::morphology_ex(&tmp_mask, &mut mask, imgproc::MORPH_OPEN, &se2, Point::new(-1, -1), 1, BORDER_CONSTANT, imgproc::morphology_default_border_value().unwrap()).unwrap();

						imgproc::find_contours(&mut mask, &mut cnts, imgproc::RETR_EXTERNAL, imgproc::CHAIN_APPROX_TC89_KCOS, Point::new(0, 0)).unwrap();

						imgproc::draw_contours(&mut frame, &cnts, -1, Scalar::new(0., 0., 255., 0.), 5, LINE_8, &no_array().unwrap(), i32::max_value(), Point::new(0, 0)).unwrap();
						cnts.shrink_to_fit();

						let cnts_sorted = cnts.iter().sorted_by(|a, b| contour_area_simple(a).partial_cmp(&contour_area_simple(b)).unwrap_or(Ordering::Equal));

						let mut filtered_cnts = VectorOfVectorOfPoint::new();
						let mut cnt_found_count = 0;
						for cnt in cnts_sorted.rev() {
							let mut hull = opencv::types::VectorOfPoint::new();
							imgproc::convex_hull(&cnt, &mut hull, false, false).unwrap();

							let area = imgproc::contour_area(&hull, false).unwrap();
							let percent_area = (area as f64 / FRAME_SIZE as f64) * 100.;
							
							if cnt_area_reject(percent_area) {
								filtered_cnts.push(cnt);
								cnt_found_count += 1;	
							}

							if cnt_found_count >= MAX_CONTOURS {
								break
							}
						}

						imgproc::draw_contours(&mut frame, &filtered_cnts, -1, Scalar::new(255., 0., 255., 0.), 5, LINE_8, &no_array().unwrap(), i32::max_value(), Point::new(0, 0)).unwrap();

						let mut super_filtered_cnts = VectorOfVectorOfPoint::new();
						if cnt_found_count == MAX_CONTOURS {
							for cnt in filtered_cnts.iter() {
								let rect = imgproc::min_area_rect(&cnt).unwrap();
								let rect_area = rect.size().unwrap().area();
							
								let area = imgproc::contour_area(&cnt, false).unwrap();
														
								let percent_filled = (area as f64 / rect_area as f64) * 100.;
																				
								if percent_filled_reject(percent_filled) {
									super_filtered_cnts.push(cnt);
								}
							}
						}
						
					
						imgproc::draw_contours(&mut frame, &super_filtered_cnts, -1, Scalar::new(255., 0., 0., 0.), 5, LINE_8, &no_array().unwrap(), i32::max_value(), Point::new(0, 0)).unwrap();
						
						let jpeg = to_jpeg(&mut frame, &params);
						io::stdout().write_all(&jpeg).unwrap();
						io::stdout().flush().unwrap();
					},
					false => {
						if cam_num == 0 {
							cam_num = 1;
						}
						else {
							cam_num = 0;
						}
						cap = open_camera(cam_num);
					},
				},
				Err(_) => {
					if cam_num == 0 {
						cam_num = 1;
					}
					else {
						cam_num = 0;
					}
					cap = open_camera(cam_num);
				},
			}
		}
	});

	//Web Server Init
	let listener = TcpListener::bind("0.0.0.0:5808").unwrap();

	for stream in listener.incoming() {
		let stream = stream.unwrap();

		handle_connection(stream);
	}

	//Make Sure Vision Thread Does Not Stop
	handle_vision.join().unwrap();
}

fn handle_connection(mut stream: TcpStream) {
	let mut buffer = [0; 512];

	stream.read(&mut buffer).unwrap();

	let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", INDEX);
	
	stream.write(response.as_bytes()).unwrap();
	stream.flush().unwrap();
}

fn open_camera(cam_num: i32) -> VideoCapture {
	let mut cap = VideoCapture::default().unwrap();
	cap.open(cam_num).unwrap();
	cap.set(4, HEIGHT as f64).unwrap();
	cap.set(3, WIDTH as f64).unwrap();
	cap.set(5, 30.).unwrap();
	cap
}

fn to_jpeg(mut frame: & Mat, params: &VectorOfint) -> std::vec::Vec<u8> {
	let mut jpeg = Vector::new();
	imencode(".jpg", &mut frame, &mut jpeg, &params).unwrap();
	jpeg.to_vec()
}

fn contour_area_simple(contour: &dyn ToInputArray) -> f64 {
	imgproc::contour_area(contour, false).unwrap()
}

fn cnt_area_reject(area: f64) -> bool {
	area > CNT_MIN_AREA && area < CNT_MAX_AREA
}

fn percent_filled_reject(percent_filled: f64) -> bool {
	percent_filled > PERCENT_FILLED_MIN && percent_filled < PERCENT_FILLED_MAX
}