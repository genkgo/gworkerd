extern crate iron;
extern crate chrono;
extern crate router;
extern crate rustc_serialize;

use self::chrono::UTC;
use self::iron::{Iron, Request, Response, IronResult};
use self::iron::status;
use self::router::Router;
use self::rustc_serialize::json;
use record_backend::RecordRepository;
use config;

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Config {
  address: String
}

pub struct HttpServer {
  config: Config,
  backend: Box<RecordRepository>,
  started_at: chrono::DateTime<UTC>
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
struct ServerResponse {
  ip: String,
  hostname: String,
  version: String,
  started_at: String
}

impl HttpServer {
  pub fn new (config: Config, backend: Box<RecordRepository>) -> HttpServer {
    HttpServer { config: config, backend: backend, started_at: UTC::now() }
  }

  pub fn listen (&mut self) {
    let mut router = Router::new();

    {
      let hostname = self.config.address.clone();
      let version = String::from(config::VERSION);
      let started_at = self.started_at.to_rfc3339().clone();

      router.get("/api/server", move |req: &mut Request| {
        let ip = format!("{}", req.local_addr).to_string();
        let server = ServerResponse { ip: ip, hostname: hostname.clone(), version: version.clone(), started_at: started_at.clone() };
        let response : String = json::encode(&server).unwrap();

        Ok(Response::with((status::Ok, response)))
      });
    }

    let address: &str = &self.config.address[..];
    Iron::new(router).http(&address).unwrap();
  }
}