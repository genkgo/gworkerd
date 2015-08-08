extern crate stomp;
extern crate rustc_serialize;

use self::stomp::session::Session;
use self::stomp::session_builder::SessionBuilder;
use self::stomp::connection::Credentials;
use self::stomp::frame::Frame;
use self::stomp::header::{Header, SuppressedHeader};
use self::stomp::subscription::AckOrNack::Ack;
use self::stomp::subscription::AckMode;
use self::rustc_serialize::json;
use std::mem::replace;
use worker::Request;

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct StompConfig {
	address: String,
	port: u16,
	host: String,
	username: String,
	password: String,
	topic: String,
	prefetch_count: u16
}

pub struct Consumer<'a> {
	session : Session<'a>,
	topic: String,
	prefetch_count: u16,
	handlers: Vec<Box<Fn(Request)>>
}

impl <'a> Consumer<'a> {

	pub fn new (config: &'a StompConfig) -> Consumer<'a> {
		let mut session = match SessionBuilder::new(&config.address, config.port)
		.with(Credentials(&config.username, &config.password))
		.with(SuppressedHeader("host"))
		.with(Header::new("host", &config.host))
		.start() {
				Ok(session) => session,
				Err(error)  => panic!("Could not connect to the server: {}", error)
			}
		;

		Consumer {
			session: session,
			topic: config.topic.clone(),
			prefetch_count: config.prefetch_count,
			handlers: vec![]
		}
	}

	pub fn subscribe<T>(&mut self, handler: T) where T : Fn(Request) + 'static {
		self.handlers.push(Box::new(handler));
	}

	pub fn listen (&mut self) {
		let handlers = replace(&mut self.handlers, Vec::new());

		self.session.subscription(&self.topic, move |frame: &Frame| {
			// deserialize received message from message queue
			let frame_body = String::from_utf8(frame.body.clone()).ok().expect("cannot convert frame body to string");
			let request: Request = json::decode(&frame_body).unwrap();

			// call handlers
			for handler in &handlers {
				handler(request.clone());
			}

			// let the server know we processed the request
			Ack
		})
			.with(AckMode::ClientIndividual)
			.with(Header::new("prefetch-count", &self.prefetch_count.to_string()))
			.start().ok().expect("unable to start receiving messages")
		;
		self.session.listen().ok().expect("unable to listen"); // Loops infinitely, awaiting messages
	}

}