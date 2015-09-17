extern crate docopt;
extern crate flexi_logger;
#[macro_use]
extern crate log;
extern crate rustc_serialize;

use config::Config;
use docopt::Docopt;
use flexi_logger::{detailed_format,init};
use flexi_logger::LogConfig as FlexiLogConfig;
use rustc_serialize::json;
use std::fs::File;
use std::io::Read;
use std::process;
use std::sync::mpsc::channel;
use std::thread;
use worker::{Request, Response, Item};
use consumer::{StompConsumer, Consumer};
use record_backend::{RecordRepository, RecordRepositoryError};
use processor::Processor;
use monitor::HttpServer;

mod config;
mod consumer;
mod processor;
mod worker;
mod record_backend;
mod monitor;

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

fn main() {
  // docopt
  let args: Args = Docopt::new(USAGE)
    .and_then(|d| d.decode())
    .unwrap_or_else(|e| e.exit());

  if args.flag_version {
    println!("Gworkerd Version: {}", config::VERSION);
    process::exit(1);
  }

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
  let (record_store_tx, record_store_rx) = channel::<Item>();

  for thread_number in 0..config.threads {
    let stomp_config = config.stomp.clone();
    let thread_tx = record_store_tx.clone();
    let processor = thread::spawn(move || {
      // connect to message queue
      let stomp_config = stomp_config.clone();
      let mut consumer = StompConsumer::new(&stomp_config);
      {
        let subscription_tx = thread_tx.clone();
        consumer.subscribe(move |request: Request| {
          info!("[{:?}] executing {} in cwd {}", request.id, request.program, request.cwd);

          let response : Response = Processor::run(request.clone());

          info!("[{:?}] finished with status: {:?}", request.id, response.status);
          debug!("[{:?}] finished with stdout: {:?}", request.id, response.stdout);
          debug!("[{:?}] finished with stderr: {:?}", request.id, response.stderr);

          let item = worker::Item { request: request.clone(), response: response };
          subscription_tx.send(item.clone()).unwrap();
        });
      }
      info!("start listening for thread {:?}", thread_number);
      consumer.listen();
    });
    threads.push(processor);
  }

  let record_connection = config.mysql.to_connection();

  {
    let connection = record_connection.clone();
    let record_store_thread = thread::spawn(move || {
      loop {
        let item = record_store_rx.recv().unwrap();
        match connection.store(item.to_record()) {
          Err(RecordRepositoryError::CannotStoreRecord) => {
            error!("[{:?}] cannot add record to result backend", item.request.id);
          },
          Err(_) => {
            error!("[{:?}] unknown error occured", item.request.id);
          },
          Ok(_) => {
            info!("[{:?}] added to result backend", item.request.id);
          }
        }
      }
    });
    threads.push(record_store_thread);
  }

  {
    let connection = record_connection.clone();
    let monitor_thread = thread::spawn(move || {
      let mut http_server = HttpServer::new(config.monitor.clone(), connection);
      http_server.listen();
    });
    threads.push(monitor_thread);
  }

  for item in threads {
    item.join().ok().expect("unable to join processor thread");
  }

}