extern crate slack;

use slack::{Event, RtmClient};

struct MyHandler;

#[allow(unused_variables)]
impl slack::EventHandler for MyHandler {
  fn on_event(&mut self, cli: &RtmClient, event: Event) {
    println!("on_event(event: {:?})", event);
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
  let mut handler = MyHandler;
  let r = RtmClient::login_and_run(&api_key, &mut handler);
  match r {
    Ok(_) => {}
    Err(err) => panic!("Error: {}", err),
  }
}
