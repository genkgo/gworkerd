extern crate flexi_logger;
#[macro_use]
extern crate log;
extern crate mysql;
extern crate stomp;
extern crate rustc_serialize;

use flexi_logger::{detailed_format,init,LogConfig};
use mysql::conn::MyOpts;
use mysql::conn::pool::MyPool;
use rustc_serialize::json;
use std::default::Default;
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
struct WorkerRequest {
	id: String,
	command: String,
	cwd: String,
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
struct WorkerResponse {
	status: String,
	stderr: String,
	stdout: String
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
struct WorkerItem {
	request: WorkerRequest,
	response: WorkerResponse
}

fn main() {
	init( LogConfig {
		log_to_file: true,
		directory: Some("/var/log/gworkerd".to_string()),
		format: detailed_format,
		.. LogConfig::new()
		},
		Some("gworkerd=info,mysql=warn,stomp=warn".to_string())
	).unwrap_or_else(|e|{panic!("Logger initialization failed with {}",e)});

	let mut threads = vec![];
	let (result_backend_tx, result_backend_rx) = channel::<WorkerItem>();

	for thread_number in 0..NUMBER_OF_THREADS {
		let tx = result_backend_tx.clone();
		let processor = thread::spawn(move || {
			// connect to message queue
			let mut session = match stomp::session("172.17.0.6", 61613)
				.with(Credentials("guest", "guest"))
				.with(SuppressedHeader("host"))
				.with(Header::new("host", "/"))
				.start() {
					Ok(session) => session,
					Err(error)  => panic!("Could not connect to the server: {} {}", error, thread_number)
				}
			;

			info!("started session {:?}", thread_number);

			session.subscription(TOPIC, |frame: &Frame| {
				// deserialize received message from message queue
				let thread_number = thread_number.clone();
				let frame_body = String::from_utf8(frame.body.clone()).ok().expect("cannot convert frame body to string");
				let request: WorkerRequest = json::decode(&frame_body).unwrap();

				// starting process
				info!("Executing {} for cwd {} in thread {}", request.command, request.cwd, thread_number);
				let output = Command::new("sh")
					.arg("-c")
					.arg(request.command.clone())
					.current_dir(request.cwd.clone())
					.output()
					.unwrap_or_else(|e| {
						panic!("failed to execute process: {}", e);
					});

				info!("received status: {:?}", &output.status.code().unwrap());
				debug!("received from stdout: {:?}", String::from_utf8_lossy(&output.stdout));
				debug!("received from stderr: {:?}", String::from_utf8_lossy(&output.stderr));

				let stdout = output.stdout;
				let stderr = output.stderr;

				// create response
				let response = WorkerResponse {
					stderr: String::from_utf8(stderr.clone()).ok().expect("cannot convert stderr to string"),
					stdout: String::from_utf8(stdout.clone()).ok().expect("cannot convert stdout to string"),
					status: output.status.code().unwrap().to_string()
				};

				// create item pair
				let item = WorkerItem { request: request, response: response };

				// save result into result backend
				tx.send(item.clone()).unwrap();

				// let the server know we processed the request
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

	let result_backend = thread::spawn(move || {
		let opts = MyOpts {
			tcp_addr: Some("172.17.42.1".to_string()),
			user: Some("root".to_string()),
			pass: Some("".to_string()),
			db_name: Some("worker_results".to_string()),
			..Default::default()
		};
		let pool = MyPool::new(opts).unwrap();

		loop {
			info!("started result backend thread, waiting for message");
			let worker = result_backend_rx.recv().unwrap();
			let mut stmt = pool.prepare(r"INSERT INTO results (id, command, cwd, status, stderr, stdout) VALUES (?, ?, ?, ?, ?, ?)").unwrap();
			stmt.execute((&worker.request.id, &worker.request.command, &worker.request.cwd, &worker.response.status, &worker.response.stderr, &worker.response.stdout)).unwrap();
			info!("added worker {:?} to result backend", &worker.request.id);
		}
	});

	for item in threads {
		item.join().ok().expect("unable to join processor thread");
	}

	result_backend.join().ok().expect("unable to join result backend thread");
}