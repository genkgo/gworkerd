extern crate chrono;
extern crate hyper;
extern crate iron;
extern crate mount;
extern crate router;
extern crate rustc_serialize;
extern crate staticfile;
extern crate urlencoded;

use self::chrono::UTC;
use self::hyper::header::ContentType;
use self::hyper::mime::{Mime, TopLevel, SubLevel};
use self::iron::prelude::*;
use self::iron::status;
use self::mount::Mount;
use self::router::Router;
use self::rustc_serialize::json;
use self::rustc_serialize::json::{ToJson, Json};
use self::staticfile::Static;
use self::urlencoded::UrlEncodedBody;
use record_backend::{RecordRepository, RecordRepositoryError};
use config;
use std::any::Any;
use std::collections::BTreeMap;
use std::path::Path;
use worker::Record;

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Config {
  address: String,
  webapp_path: String,
  websockets: bool,
  password: String
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
  started_at: String,
  websockets: bool
}

struct JobsResponse {
  job: Vec<Record>
}

impl ToJson for JobsResponse {

  fn to_json(&self) -> Json {
    let mut data = BTreeMap::new();
    data.insert("job".to_string(), self.job.to_json());
    Json::Object(data)
  }

}

impl <R: RecordRepository + Clone + Send + Sync + Any> HttpServer<R> {

  pub fn new (config: Config, backend: R) -> HttpServer <R> {
    HttpServer { config: config, backend: backend, started_at: UTC::now() }
  }

  pub fn listen (&mut self) {
    let mut router = Router::new();
    let mut mount = Mount::new();

    {
      let hostname = self.config.address.clone();
      let version = String::from(config::VERSION);
      let started_at = self.started_at.to_rfc3339().clone();
      let websockets = self.config.websockets;

      router.get("/api/server", move |req: &mut Request| {
        let ip = format!("{}", req.local_addr).to_string();
        let server = ServerResponse {
          ip: ip, hostname: hostname.clone(), version: version.clone(), started_at: started_at.clone(), websockets: websockets
        };
        let response_data : String = json::encode(&server).unwrap();
        let mut response = Response::with((status::Ok, response_data));
        response.headers.set(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])));
        Ok(response)
      });
    }

    {
      let password = self.config.password;
      router.post("/api/auth", move |req: &mut Request| {
        match req.get_ref::<UrlEncodedBody>() {
          Ok(ref body) => {
            if !body.contains_key("password") {
              return Ok(Response::with((status::BadRequest, "")))
            }
            if body.get("password").unwrap()[0] != password {
              return Ok(Response::with((status::Unauthorized, "")))
            }
            Ok(Response::with((status::Ok, "")))
          },
          Err(_) => {
            Ok(Response::with((status::BadRequest, "")))
          }
        };
      });
    }

    {
      let backend = self.backend.clone();
      router.get("/api/jobs", move |req: &mut Request| {
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
            let job_response = JobsResponse { job: records };
            let response_data = job_response.to_json();

            let mut response = Response::with((status::Ok, response_data.to_string()));
            response.headers.set(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])));
            Ok(response)
          }
        }
      });
    }
    {
      let backend = self.backend.clone();
      router.get("/api/jobs/:id", move |req: &mut Request| {
        let ref id = req.extensions.get::<Router>().unwrap().find("id").unwrap_or("/");
        match backend.fetch_record(id.to_string()) {
          Err(RecordRepositoryError::CannotDenormalizeRecord) => {
            let json = "{\"message\": \"Cannot denormalize records from database\"}";
            let mut response = Response::with((status::InternalServerError, json));
            response.headers.set(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])));
            Ok(response)
          },
          Err(RecordRepositoryError::RecordNotFound) => {
            let json = "{\"message\": \"Record does not exists\"}";
            let mut response = Response::with((status::NotFound, json));
            response.headers.set(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])));
            Ok(response)
          },
          Err(_) => {
            let mut response = Response::with((status::InternalServerError, "{\"message\": \"Unknown error\" }"));
            response.headers.set(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])));
            Ok(response)
          },
          Ok(record) => {
            let job_response = JobsResponse { job: vec![record] };
            let response_data = job_response.to_json();

            let mut response = Response::with((status::Ok, response_data.to_string()));
            response.headers.set(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])));
            Ok(response)
          }
        }
      });
    }

    let mut mount = Mount::new();
    mount.mount("/", router);
    mount.mount("/monitor", Static::new(Path::new(&self.config.webapp_path)));

//    let address: &str = &self.config.address[..];
    Iron::new(mount).http(&*self.config.address).unwrap();
  }
}