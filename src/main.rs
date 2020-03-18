extern crate slack;

use slack::{Event, Message, RtmClient};

struct PMHandler;

#[allow(unused_variables)]
impl slack::EventHandler for PMHandler {
  fn on_event(&mut self, cli: &RtmClient, event: Event) {
    println!("on_event(event: {:?})", event);
    let slack_message = match event {
      Event::Message(m) => Box::leak(m),
      _ => return,
    };
    let m = match &slack_message {
      Message::Standard(m) => m,
      _ => return,
    };
    let text = match &m.text {
      Some(t) => t,
      None => return,
    };
    let user = match &m.user {
      Some(u) => u,
      None => return,
    };
    println!("{}: '{}'", user, text);
    // TODO: store the message here
    // TODO: if text == "done" then post the collected messages to the channel:
    // let channel_id = "DND47PSF9";
    // let _ = cli.sender().send_message(&channel_id, "Recorded, thanks!");
  }

  fn on_close(&mut self, cli: &RtmClient) {
    println!("on_close");
    //TODO: set offline signal
  }

  fn on_connect(&mut self, cli: &RtmClient) {
    println!("on_connect");
    //TODO: set online signal
  }
}

fn main() {
  let args: Vec<String> = std::env::args().collect();
  let api_key = match args.len() {
    0 | 1 => panic!("No api-key in args! Usage: cargo run -- <api-key>"),
    x => args[x - 1].clone(),
  };
  let mut handler = PMHandler;
  let r = RtmClient::login_and_run(&api_key, &mut handler);
  match r {
    Ok(_) => {}
    Err(err) => panic!("Error: {}", err),
  }
}
