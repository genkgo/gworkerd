extern crate stomp;
extern crate rustc_serialize;

use rustc_serialize::json;
use std::process::Command;
use std::sync::mpsc::channel;
use std::thread;
use stomp::frame::Frame;
use stomp::header::{Header, SuppressedHeader};
use stomp::connection::{Credentials};
use stomp::subscription::AckOrNack::Ack;
use stomp::subscription::AckMode;

const NUMBER_OF_THREADS : u32 = 10;
const TOPIC : &'static str = "/queue/test_messages";

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
struct WorkerItem {
	id: String,
	command: String,
	cwd: String
}

fn main() {
	let mut threads = vec![];
	let (result_backend_tx, result_backend_rx) = channel::<WorkerItem>();

	for thread_number in 0..NUMBER_OF_THREADS {
		let tx = result_backend_tx.clone();
		let processor = thread::spawn(move || {
			let mut session = match stomp::session("172.17.0.4", 61613)
				.with(Credentials("guest", "guest"))
				.with(SuppressedHeader("host"))
				.with(Header::new("host", "/"))
				.start() {
					Ok(session) => session,
					Err(error)  => panic!("Could not connect to the server: {} {}", error, thread_number)
				}
			;

			println!("started session {:?}", thread_number);

			session.subscription(TOPIC, |frame: &Frame| {
				let thread_number = thread_number.clone();
				let frame_body = String::from_utf8(frame.body.clone()).ok().expect("cannot convert frame body to string");
				let worker: WorkerItem = json::decode(&frame_body).unwrap();

				println!("Will execute {} for cwd {} in thread {}", worker.command, worker.cwd, thread_number);
				let output = Command::new("sh")
					.arg("-c")
					.arg(worker.command.clone())
					.current_dir(worker.cwd.clone())
					.output()
					.unwrap_or_else(|e| {
						panic!("failed to execute process: {}", e);
					});
					println!("{:?}", String::from_utf8_lossy(&output.stdout));
					println!("{:?}", &output.status.code().unwrap());
					tx.send(worker.clone()).unwrap();
					Ack
				})
				.with(AckMode::ClientIndividual)
				.with(Header::new("prefetch-count", "1"))
				.start().ok().expect("unable to start receiving messages")
			;
			session.listen().ok().expect("unable to listen"); // Loops infinitely, awaiting messages
		});
		threads.push(processor);
	}

	for item in threads {
		item.join().ok().expect("unable to join processor thread");
	}

	let result_backend = thread::spawn(move || {
		let worker = result_backend_rx.recv().unwrap();
		println!("Processed message {}", worker.command);
	});

	result_backend.join().ok().expect("unable to join result backend thread");
}