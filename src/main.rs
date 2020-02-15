use opencv::prelude::*;
use opencv::videoio::VideoCapture;
use opencv::imgcodecs::*;
use opencv::types::*;
use opencv::imgproc;
use opencv::core::{self, Scalar, Size, Point};

use std::io::{self, Write};
use std::io::prelude::*;
use std::thread;
use std::net::TcpListener;
use std::net::TcpStream;

const INDEX: &str = include_str!("site/index.html");

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
						core::in_range(&hsv, &Scalar::new(50., 100., 70., 0.), &Scalar::new(70., 255., 255., 0.), &mut mask).unwrap();

						let se1 = imgproc::get_structuring_element(imgproc::MORPH_RECT, Size::new(4, 4), Point::new(-1, -1)).unwrap();
						let se2 = imgproc::get_structuring_element(imgproc::MORPH_RECT, Size::new(4, 4), Point::new(-1, -1)).unwrap();

						let mut tmp_mask = Mat::default().unwrap();
						
						imgproc::morphology_ex(&mask, &mut tmp_mask, imgproc::MORPH_CLOSE, &se1, Point::new(-1, -1), 1, core::BORDER_CONSTANT, imgproc::morphology_default_border_value().unwrap()).unwrap();
						imgproc::morphology_ex(&tmp_mask, &mut mask, imgproc::MORPH_OPEN, &se2, Point::new(-1, -1), 1, core::BORDER_CONSTANT, imgproc::morphology_default_border_value().unwrap()).unwrap();

						imgproc::find_contours(&mut mask, &mut cnts, imgproc::RETR_EXTERNAL, imgproc::CHAIN_APPROX_TC89_KCOS, Point::new(0, 0)).unwrap();

						imgproc::draw_contours(&mut frame, &cnts, -1, Scalar::new(0., 0., 255., 0.), 5, core::LINE_8, &core::no_array().unwrap(), i32::max_value(), Point::new(0, 0)).unwrap();
						cnts.shrink_to_fit();

						for cnt in cnts.iter() {
							
						}
						
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
	cap.set(4, 480.).unwrap();
	cap.set(3, 640.).unwrap();
	cap.set(5, 30.).unwrap();
	cap
}

fn to_jpeg(mut frame: & Mat, params: &VectorOfint) -> std::vec::Vec<u8> {
	let mut jpeg = Vector::new();
	imencode(".jpg", &mut frame, &mut jpeg, &params).unwrap();
	jpeg.to_vec()
}
