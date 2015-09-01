extern crate docopt;
extern crate flexi_logger;
#[macro_use]
extern crate log;
extern crate rustc_serialize;

use docopt::Docopt;
use flexi_logger::{detailed_format,init};
use flexi_logger::LogConfig as FlexiLogConfig;
use rustc_serialize::json;
use std::fs::File;
use std::io::Read;
use std::sync::mpsc::channel;
use std::thread;
use worker::{Request, Response, Item};
use consumer::{StompConfig, StompConsumer, Consumer};
use result_backend::{MysqlConfig, ResultBackend, ResultBackendError};
use processor::Processor;

mod consumer;
mod processor;
mod worker;
mod result_backend;

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

  let result_connection = config.mysql.to_connection();
  let result_thread = thread::spawn(move || {
    let connection = result_connection.clone();
    loop {
      let item = result_backend_rx.recv().unwrap();
      match connection.store(item.to_record()) {
        Err(ResultBackendError::CannotStoreResult) => {
          error!("[{:?}] cannot add to result backend", item.request.id);
        },
        Ok(_) => {
          info!("[{:?}] added to result backend", item.request.id);
        }
      }

    }
  });

  for item in threads {
    item.join().ok().expect("unable to join processor thread");
  }

  result_thread.join().ok().expect("unable to join result backend thread");
}