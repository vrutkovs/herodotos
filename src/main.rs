use slack;
use std::collections::HashMap;

struct MsgData {
  user: String,
  text: String,
}

struct PMHandler {
  daily_statuses: HashMap<String, Vec<String>>,
}

#[allow(unused_variables)]
impl slack::EventHandler for PMHandler {
  fn on_event(&mut self, cli: &slack::RtmClient, event: slack::Event) {
    println!("on_event(event: {:?})", event);
    let msg_data = match self.get_message(event) {
      Some(m) => m,
      None => return,
    };
    println!("{}: '{}'", msg_data.user, msg_data.text);

    // if text == "done" then post the collected messages to the channel:
    //
    //

    match self.process_message(msg_data) {
      Some(message) => {
        let channel_id = "DND47PSF9";
        let _ = cli.sender().send_message(&channel_id, message.as_str());
      }
      None => return,
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

impl PMHandler {
  fn get_message(&mut self, event: slack::Event) -> Option<MsgData> {
    let slack_message = match event {
      slack::Event::Message(m) => Box::leak(m),
      _ => return None,
    };
    let m = match &slack_message {
      slack::Message::Standard(m) => m,
      _ => return None,
    };
    let text = match &m.text {
      Some(t) => t,
      None => return None,
    };
    let user = match &m.user {
      Some(u) => u,
      None => return None,
    };
    Some(MsgData {
      text: text.to_string(),
      user: user.to_string(),
    })
  }

  fn process_message(&mut self, msg: MsgData) -> Option<String> {
    match msg.text.as_str() {
      "done" => Some(format!("{:?}", self.daily_statuses)),
      _ => {
        // Store messages
        match self.daily_statuses.get_mut(&msg.user) {
          Some(usr_status) => {
            usr_status.push(msg.text.to_string());
          }
          None => {
            self
              .daily_statuses
              .insert(msg.user.to_string(), vec![msg.text.to_string()]);
          }
        }
        None
      }
    }
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
