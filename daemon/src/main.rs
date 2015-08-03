extern crate docopt;
extern crate flexi_logger;
#[macro_use]
extern crate log;
extern crate mysql;
extern crate stomp;
extern crate rustc_serialize;

use docopt::Docopt;
use flexi_logger::{detailed_format,init};
use flexi_logger::LogConfig as FlexiLogConfig;
use mysql::conn::MyOpts;
use mysql::conn::pool::MyPool;
use rustc_serialize::json;
use std::default::Default;
use std::fs::File;
use std::io::Read;
use std::process::Command;
use std::sync::mpsc::channel;
use std::thread;
use stomp::connection::{Credentials};
use stomp::frame::Frame;
use stomp::header::{Header, SuppressedHeader};
use stomp::subscription::AckOrNack::Ack;
use stomp::subscription::AckMode;

// Write the Docopt usage string.
static USAGE: &'static str = "
Usage: gworkerd [options] <config>
       gworkerd (--help | --version)

Options:
    -h, --help     Print this information
    -v, --version  Show the version.
";

#[derive(RustcDecodable, Debug)]
struct Args {
	arg_config: String,
	flag_help: bool,
	flag_version: bool,
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
struct StompConfig {
	address: String,
	port: u16,
	host: String,
	username: String,
	password: String,
	topic: String,
  prefetch_count: u16
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
struct MysqlConfig {
	address: String,
	username: String,
	password: String,
	database: String
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
struct LogConfig {
	directory: String,
	levels: String
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
struct Config {
	log: LogConfig,
	threads: u32,
	stomp: StompConfig,
	mysql: MysqlConfig,
}

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
	// docopt
	let args: Args = Docopt::new(USAGE)
		.and_then(|d| d.decode())
		.unwrap_or_else(|e| e.exit());

	let mut file = File::open(args.arg_config).unwrap();
	let mut data = String::new();
	file.read_to_string(&mut data).unwrap();
	let config: Config = json::decode(&data).unwrap();

	init( FlexiLogConfig {
		log_to_file: true,
		directory: Some(config.log.directory.clone()),
		format: detailed_format,
		.. FlexiLogConfig::new()
		},
		Some(config.log.levels.clone())
	).unwrap_or_else(|e|{panic!("Logger initialization failed with {}",e)});

	let mut threads = vec![];
	let (result_backend_tx, result_backend_rx) = channel::<WorkerItem>();

	for thread_number in 0..config.threads {
		let tx = result_backend_tx.clone();
		let stomp_config = config.stomp.clone();
		let processor = thread::spawn(move || {
			// connect to message queue
			let stomp_config = stomp_config.clone();
			let mut session = match stomp::session(&stomp_config.address, stomp_config.port)
				.with(Credentials(&stomp_config.username, &stomp_config.password))
				.with(SuppressedHeader("host"))
				.with(Header::new("host", &stomp_config.host))
				.start() {
					Ok(session) => session,
					Err(error)  => panic!("Could not connect to the server: {} {}", error, thread_number)
				}
			;

			info!("started session {:?}", thread_number);

			session.subscription(&stomp_config.topic, |frame: &Frame| {
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
				.with(Header::new("prefetch-count", &stomp_config.prefetch_count.to_string()))
				.start().ok().expect("unable to start receiving messages")
			;
			session.listen().ok().expect("unable to listen"); // Loops infinitely, awaiting messages
		});
		threads.push(processor);
	}

	let result_backend = thread::spawn(move || {
		let mysql_config = config.mysql.clone();
		let opts = MyOpts {
			tcp_addr: Some(mysql_config.address),
			user: Some(mysql_config.username),
			pass: Some(mysql_config.password),
			db_name: Some(mysql_config.database.to_string()),
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