#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Request {
  pub id: String,
  pub command: String,
  pub cwd: String,
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Response {
  pub status: String,
  pub stderr: String,
  pub stdout: String
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Item {
  pub request: Request,
  pub response: Response
}