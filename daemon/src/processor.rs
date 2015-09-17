extern crate chrono;

use std::process::Command;
use worker::{Request, Response, Period};
use self::chrono::UTC;

pub struct Processor;

impl Processor {

  pub fn run (request: Request) -> Response {
    // starting process
    let started_at = UTC::now();

    let mut command = Command::new("sh");
    command.arg("-c");
    command.arg(request.command.clone());
    command.current_dir(request.cwd.clone());

    match command.output() {
      Err(_) => {
        let finished_at = UTC::now();

        // create response
        Response {
          stderr: "could not execute".to_string(),
          stdout: "".to_string(),
          status: 1i32,
          period: Period {
            started_at: started_at,
            finished_at: finished_at
          }
        }
      },
      Ok(output) => {
        let stdout = output.stdout;
        let stderr = output.stderr;
        let finished_at = UTC::now();

        // create response
        Response {
          stderr: String::from_utf8(stderr.clone()).ok().expect("cannot convert stderr to string"),
          stdout: String::from_utf8(stdout.clone()).ok().expect("cannot convert stdout to string"),
          status: output.status.code().unwrap(),
          period: Period {
          started_at: started_at,
          finished_at: finished_at
          }
        }
      }
    }
  }

}