use opencv::prelude::*;
use opencv::videoio::VideoCapture;
use opencv::imgcodecs::*;
use opencv::types::*;

use std::io::{self, Write};
use std::io::prelude::*;
use std::thread;
use std::net::TcpListener;
use std::net::TcpStream;
use std::fs;

const INDEX: &str = include!("site/index.html");

fn main() {
	//Vision Thread
	let handle_vision = thread::spawn(|| {
		//Open Camera
		let mut cap = open_camera(0);
		//Init Params For Encoding
		let params: VectorOfint = VectorOfint::new();
		//Init Captured Frmae
		let mut frame: Mat = Mat::default().unwrap();
		//Camera Number To Check
		let mut cam_num = 0;
	
		//Main Vision Loop
		loop {
			match cap.read(&mut frame) {
				Ok(cap_read) => match cap_read {
					true => {
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

	let contents = fs::read_to_string("site/index.html").unwrap();

	let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", contents);
	
	stream.write(response.as_bytes()).unwrap();
	stream.flush().unwrap();
}

fn open_camera(cam_num: i32) -> VideoCapture {
	let mut cap = VideoCapture::default().unwrap();
	cap.open(cam_num).unwrap();
	cap
}

fn to_jpeg(mut frame: & Mat, params: &VectorOfint) -> std::vec::Vec<u8> {
	let mut jpeg = Vector::new();
	imencode(".jpg", &mut frame, &mut jpeg, &params).unwrap();
	jpeg.to_vec()
}
