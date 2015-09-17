extern crate chrono;
extern crate rustc_serialize;

use self::chrono::{DateTime, UTC};
use rustc_serialize::json::{ToJson, Json};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Period {
  pub started_at: chrono::DateTime<UTC>,
  pub finished_at: chrono::DateTime<UTC>
}

#[derive(Debug, Clone, RustcDecodable)]
pub struct Request {
  pub id: String,
  pub command: String,
  pub cwd: String
}

#[derive(Debug, Clone)]
pub struct Response {
  pub status: i32,
  pub stderr: String,
  pub stdout: String,
  pub period: Period
}

#[derive(Debug, Clone)]
pub struct Item {
  pub request: Request,
  pub response: Response
}

impl Item {
  pub fn to_record (&self) -> Record {
    Record {
      id: self.request.id.clone(),
      command: self.request.command.clone(),
      cwd: self.request.cwd.clone(),
      status: self.response.status.clone(),
      stderr: self.response.stderr.clone(),
      stdout: self.response.stdout.clone(),
      started_at: self.response.period.started_at.clone(),
      finished_at: self.response.period.finished_at.clone()
    }
  }
}

#[derive(Debug, Clone)]
pub struct Record {
  pub id: String,
  pub command: String,
  pub cwd: String,
  pub status: i32,
  pub stderr: String,
  pub stdout: String,
  pub started_at: chrono::DateTime<UTC>,
  pub finished_at: chrono::DateTime<UTC>
}

impl ToJson for Record {

  fn to_json(&self) -> Json {
    let mut data = BTreeMap::new();
    data.insert("id".to_string(), self.id.to_json());
    data.insert("command".to_string(), self.command.to_json());
    data.insert("cwd".to_string(), self.cwd.to_json());
    data.insert("status".to_string(), self.status.to_json());
    data.insert("stderr".to_string(), self.stderr.to_json());
    data.insert("stdout".to_string(), self.stdout.to_json());
    data.insert("started_at".to_string(), self.started_at.to_rfc3339().to_json());
    data.insert("finished_at".to_string(), self.finished_at.to_rfc3339().to_json());
    Json::Object(data)
  }

}