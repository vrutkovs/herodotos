use slack;
use slack_api;
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

    let slack_message = match event {
      slack::Event::Message(m) => Box::leak(m),
      _ => return,
    };
    let m = match &slack_message {
      slack::Message::Standard(m) => {
        let msg_data = match self.get_message(m) {
          Some(m) => m,
          None => return,
        };
        println!("{}: '{}'", msg_data.user, msg_data.text);

        match self.process_message(msg_data) {
          Some(message) => self.post_status(cli, message),
          None => return,
        }
      }
      slack::Message::MessageDeleted(m) => {
        match self.get_deleted_message(m) {
          Some(m) => self.process_deleted_message(m),
          None => return,
        };
      }
      slack::Message::MessageChanged(m) => {
        match self.get_edited_message(m) {
          Some(m) => self.process_edited_message(m),
          None => return,
        };
      }
      _ => return,
    };
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
  fn get_message(&mut self, m: &slack_api::MessageStandard) -> Option<MsgData> {
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

  fn get_deleted_message(&mut self, m: &slack_api::MessageMessageDeleted) -> Option<MsgData> {
    let previous_message = match &m.previous_message {
      Some(previous_message) => previous_message,
      None => return None,
    };

    //TODO: merge with get_message?
    let text = match &previous_message.text {
      Some(t) => t,
      None => return None,
    };
    let user = match &previous_message.user {
      Some(u) => u,
      None => return None,
    };
    Some(MsgData {
      text: text.to_string(),
      user: user.to_string(),
    })
  }

  fn get_edited_message(
    &mut self,
    m: &slack_api::MessageMessageChanged,
  ) -> Option<(MsgData, MsgData)> {
    let previous_message = match &m.previous_message {
      Some(previous_message) => previous_message,
      None => return None,
    };
    let new_message = match &m.message {
      Some(new_message) => new_message,
      None => return None,
    };

    //TODO: merge with get_message?
    let previous_text = match &previous_message.text {
      Some(t) => t,
      None => return None,
    };
    let new_text = match &new_message.text {
      Some(t) => t,
      None => return None,
    };
    let user = match &previous_message.user {
      Some(u) => u,
      None => return None,
    };
    Some((
      MsgData {
        text: previous_text.to_string(),
        user: user.to_string(),
      },
      MsgData {
        text: new_text.to_string(),
        user: user.to_string(),
      },
    ))
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

  fn process_deleted_message(&mut self, msg: MsgData) {
    match self.daily_statuses.get_mut(&msg.user) {
      Some(usr_status) => {
        let index = usr_status.iter().position(|i| *i == msg.text).unwrap();
        usr_status.remove(index);
      }
      None => return,
    }
  }

  fn process_edited_message(&mut self, msgs: (MsgData, MsgData)) {
    let previous_message = msgs.0;
    let new_message = msgs.1;

    match self.daily_statuses.get_mut(&previous_message.user) {
      Some(usr_status) => {
        let index = usr_status
          .iter()
          .position(|i| *i == previous_message.text)
          .unwrap();
        let _ = std::mem::replace(&mut usr_status[index], new_message.text.to_string());
      }
      None => return,
    }
  }

  fn post_status(&mut self, cli: &slack::RtmClient, message: String) {
    let channel_id = "DND47PSF9";
    let _ = cli.sender().send_message(&channel_id, message.as_str());
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
