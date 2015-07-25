extern crate stomp;

use stomp::frame::Frame;
use stomp::subscription::AckOrNack::Ack;

fn main() {

  let destination = "/topic/messages";
  let mut message_count: u64 = 0;


  let mut session = match stomp::session("172.17.0.5", 61613).start() {
      Ok(session) => session,
      Err(error)  => panic!("Could not connect to the server: {}", error)
   };

  session.subscription(destination, |frame: &Frame| {
    message_count += 1;
    println!("Received message {}", frame);
    Ack
  }).start().ok().expect("unable to receive message");

  let send_error = "unable to send message";
  session.message(destination, "Animal").send().ok().expect(send_error);
  session.message(destination, "Vegetable").send().ok().expect(send_error);
  session.message(destination, "Mineral").send().ok().expect(send_error);

  session.listen().ok().expect("unable to listen"); // Loops infinitely, awaiting messages

  session.disconnect().ok().expect("cannot disconnect from server");
}