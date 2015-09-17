extern crate hyper;
extern crate iron;
extern crate chrono;
extern crate router;
extern crate rustc_serialize;

use self::chrono::UTC;
use self::hyper::header::{Headers, ContentType};
use self::hyper::mime::{Mime, TopLevel, SubLevel};
use self::iron::{Iron, Request, Response, IronResult};
use self::iron::status;
use self::router::Router;
use self::rustc_serialize::json;
use record_backend::{RecordRepository, RecordRepositoryError};
use config;

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Config {
  address: String
}

pub struct HttpServer<R> {
  config: Config,
  backend: R,
  started_at: chrono::DateTime<UTC>
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
struct ServerResponse {
  ip: String,
  hostname: String,
  version: String,
  started_at: String
}

impl <R: RecordRepository + Clone> HttpServer<R> {

  pub fn new (config: Config, backend: R) -> HttpServer <R> {
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
        let response_data : String = json::encode(&server).unwrap();
        let mut response = Response::with((status::Ok, response_data));
        response.headers.set(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])));
        Ok(response)
      });
    }

    {
      let backend = self.backend.clone();
      router.get("/api/jobs", |req: &mut Request| {
        match backend.fetch_limit(30, 0) {
            Err(RecordRepositoryError::CannotDenormalizeRecord) => {
            let json = "{\"message\": \"Cannot denormalize records from database\"}";
            let mut response = Response::with((status::InternalServerError, json));
            response.headers.set(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])));
            Ok(response)
          },
            Err(_) => {
            let mut response = Response::with((status::InternalServerError, "{\"message\": \"Unknown error\" }"));
            response.headers.set(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])));
            Ok(response)
          },
            Ok(records) => {
              Ok(Response::with((status::Ok, "test")))
          }
        }
      });
    }

    let address: &str = &self.config.address[..];
    Iron::new(router).http(&address).unwrap();
  }
}