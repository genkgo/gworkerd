extern crate stomp;
extern crate rustc_serialize;

use self::stomp::session::Session;
use self::stomp::session_builder::SessionBuilder;
use self::stomp::connection::{Credentials, HeartBeat};
use self::stomp::frame::Frame;
use self::stomp::header::{Header, SuppressedHeader};
use self::stomp::subscription::AckOrNack::{Ack, Nack};
use self::stomp::subscription::AckMode;
use self::rustc_serialize::json;
use std::mem::replace;
use worker::Request;

pub trait Consumer {
  fn subscribe<T>(&mut self, handler: T) where T : Fn(Request) + 'static;
  fn listen (&mut self);
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct StompConfig {
  address: String,
  port: u16,
  host: String,
  username: String,
  password: String,
  topic: String,
  prefetch_count: u16,
  heartbeat: u32
}

pub struct StompConsumer<'a> {
  session : Session<'a>,
  topic: String,
  prefetch_count: u16,
  handlers: Vec<Box<Fn(Request)>>
}

impl<'a> StompConsumer<'a> {

  pub fn new (config: &'a StompConfig) -> StompConsumer<'a> {
    let session = match SessionBuilder::new(&config.address, config.port)
      .with(Credentials(&config.username, &config.password))
      .with(SuppressedHeader("host"))
      .with(HeartBeat(config.heartbeat.clone(), config.heartbeat.clone()))
      .with(Header::new("host", &config.host))
      .start() {
        Ok(session) => session,
        Err(error)  => panic!("Could not connect to the server: {}", error)
      }
    ;

    StompConsumer {
      session: session,
      topic: config.topic.clone(),
      prefetch_count: config.prefetch_count,
      handlers: vec![]
    }
  }

}

impl<'a> Consumer for StompConsumer<'a> {

  fn subscribe<T>(&mut self, handler: T) where T : Fn(Request) + 'static {
    self.handlers.push(Box::new(handler));
  }

  fn listen (&mut self) {
    let handlers = replace(&mut self.handlers, Vec::new());

    self.session.on_before_send(|frame: &mut Frame| {
      match frame.command.as_ref() {
        "NACK" => {
          frame.headers.push(Header::new("requeue", "false"));
        },
        _ => {}
      }
    });

    self.session.subscription(&self.topic, move |frame: &Frame| {
      let frame_body = match String::from_utf8(frame.body.clone()) {
        Ok(v) => v,
        Err(_) => return Nack
      };

      let request: Request = match json::decode(&frame_body) {
        Ok(v) => v,
        Err(_) => return Nack
      };

      for handler in &handlers {
        handler(request.clone());
      }

      Ack
    })
      .with(AckMode::ClientIndividual)
      .with(Header::new("prefetch-count", &self.prefetch_count.to_string()))
      .start().ok().expect("unable to start receiving messages")
    ;
    self.session.listen().ok().expect("unable to listen");
  }
}