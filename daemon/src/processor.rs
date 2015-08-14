use std::process::Command;
use worker::{Request, Response};

pub struct Processor;

impl Processor {

  pub fn run (request: Request) -> Response {
    // starting process
    let output = Command::new("sh")
    .arg("-c")
    .arg(request.command.clone())
    .current_dir(request.cwd.clone())
    .output()
    .unwrap_or_else(|e| {
      panic!("failed to execute process: {}", e);
    });

    let stdout = output.stdout;
    let stderr = output.stderr;

    // create response
    Response {
      stderr: String::from_utf8(stderr.clone()).ok().expect("cannot convert stderr to string"),
      stdout: String::from_utf8(stdout.clone()).ok().expect("cannot convert stdout to string"),
      status: output.status.code().unwrap().to_string()
    }
  }

}