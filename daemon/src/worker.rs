#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Period {
  pub started_at: String,
  pub finished_at: String
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Request {
  pub id: String,
  pub program: String,
  pub args: Vec<String>,
  pub cwd: String
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Response {
  pub status: String,
  pub stderr: String,
  pub stdout: String,
  pub period: Period
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Item {
  pub request: Request,
  pub response: Response
}

impl Item {
  pub fn to_record (&self) -> Record {
    let mut command = self.request.program.clone();
    command.push_str(" ");

    for arg in &self.request.args {
      command.push_str(" ");
      command.push_str(arg);
    }

    Record {
      id: self.request.id.clone(),
      command: command,
      cwd: self.request.cwd.clone(),
      status: self.response.status.clone(),
      stderr: self.response.stderr.clone(),
      stdout: self.response.stdout.clone(),
      started_at: self.response.period.started_at.clone(),
      finished_at: self.response.period.finished_at.clone()
    }
  }
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Record {
  pub id: String,
  pub command: String,
  pub cwd: String,
  pub status: String,
  pub stderr: String,
  pub stdout: String,
  pub started_at: String,
  pub finished_at: String
}