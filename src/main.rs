use slack;
use std::collections::HashMap;

struct PMHandler {
  daily_statuses: HashMap<String, Vec<String>>,
}

#[allow(unused_variables)]
impl slack::EventHandler for PMHandler {
  fn on_event(&mut self, cli: &slack::RtmClient, event: slack::Event) {
    println!("on_event(event: {:?})", event);
    let slack_message = match event {
      slack::Event::Message(m) => Box::leak(m),
      _ => return,
    };
    let m = match &slack_message {
      slack::Message::Standard(m) => m,
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

    // if text == "done" then post the collected messages to the channel:
    match text.as_str() {
      "done" => {
        let channel_id = "DND47PSF9";
        let debug_output = format!("{:?}", self.daily_statuses);
        let _ = cli
          .sender()
          .send_message(&channel_id, debug_output.as_str());
      }
      _ => {
        // Store messages
        match self.daily_statuses.get_mut(user) {
          Some(usr_status) => usr_status.push(text.to_string()),
          None => {
            self
              .daily_statuses
              .insert(user.to_string(), vec![text.to_string()]);
          }
        }
      }
    }
  }

  fn on_close(&mut self, cli: &slack::RtmClient) {
    println!("on_close");
    //TODO: set offline signal
  }

  fn on_connect(&mut self, cli: &slack::RtmClient) {
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
  let daily_statuses = HashMap::new();
  let mut handler = PMHandler { daily_statuses };
  let r = slack::RtmClient::login_and_run(&api_key, &mut handler);
  match r {
    Ok(_) => {}
    Err(err) => panic!("Error: {}", err),
  }
}
