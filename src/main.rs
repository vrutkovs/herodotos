extern crate slack;

use slack::{Event, Message, RtmClient};

struct PMHandler;

#[allow(unused_variables)]
impl slack::EventHandler for PMHandler {
  fn on_event(&mut self, cli: &RtmClient, event: Event) {
    println!("on_event(event: {:?})", event);
    let m = match event {
      Event::Message(m) => Box::leak(m),
      _ => return,
    };
    let text = match &m {
      Message::Standard(m) => match &m.text {
        Some(t) => t,
        None => return,
      },
      _ => return,
    };
    println!("{}", text)
  }

  fn on_close(&mut self, cli: &RtmClient) {
    println!("on_close");
  }

  fn on_connect(&mut self, cli: &RtmClient) {
    println!("on_connect");
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
