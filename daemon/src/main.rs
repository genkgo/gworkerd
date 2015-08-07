extern crate docopt;
extern crate flexi_logger;
#[macro_use]
extern crate log;
extern crate mysql;
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
use std::sync::mpsc::channel;
use std::thread;
use consumer::{StompConfig, Consumer};
use processor::Processor;
use worker::{Request, Item};

mod consumer;
mod processor;
mod worker;

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
	let (result_backend_tx, result_backend_rx) = channel::<Item>();

	for thread_number in 0..config.threads {
		let stomp_config = config.stomp.clone();
		let thread_tx = result_backend_tx.clone();
		let processor = thread::spawn(move || {
			// connect to message queue
			let stomp_config = stomp_config.clone();
			let mut consumer = Consumer::new(&stomp_config);
			{
				let subscription_tx = thread_tx.clone();
				consumer.subscribe(move |request: Request| {
					info!("Executing {} for cwd {}", request.command, request.cwd);

					let response = Processor::run(request.clone());

					info!("received status: {:?}", &response.status);
					debug!("received from stdout: {:?}", &response.stdout);
					debug!("received from stderr: {:?}", &response.stderr);

					let item = worker::Item { request: request.clone(), response: response };
					subscription_tx.send(item.clone()).unwrap();
				});
			}
			consumer.listen();
			info!("started session {:?}", thread_number);
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
			// insert uuid's the optimized way https://www.percona.com/blog/2014/12/19/store-uuid-optimized-way/
			let worker = result_backend_rx.recv().unwrap();
			let mut stmt = pool.prepare(r"INSERT INTO results (id, command, cwd, status, stderr, stdout) VALUES (UNHEX(?), ?, ?, ?, ?, ?)").unwrap();
			let ordered_uuid = worker.request.id[14..18].to_string() + &worker.request.id[9..13].to_string() + &worker.request.id[0..8].to_string() + &worker.request.id[19..23].to_string() + &worker.request.id[24..].to_string();
			stmt.execute((&ordered_uuid, &worker.request.command, &worker.request.cwd, &worker.response.status, &worker.response.stderr, &worker.response.stdout)).unwrap();
			info!("added worker {:?} to result backend", &worker.request.id);
		}
	});

	for item in threads {
		item.join().ok().expect("unable to join processor thread");
	}

	result_backend.join().ok().expect("unable to join result backend thread");
}