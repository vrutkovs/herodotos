use slack;
use slack_api;
use std::collections::HashMap;

struct MsgData {
  user: String,
  text: String,
}

struct PMHandler {
  channel_id: String,
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

        let channel_id: String = match &m.channel {
          Some(c) => String::from(c),
          None => return,
        };
        // Ensure its a private message to the bot
        // we have to ensure that its None
        let channel = cli.start_response().channels.as_ref().and_then(|channels| {
          channels.iter().find(|chan| match chan.id {
            None => false,
            Some(ref id) => *id == channel_id,
          })
        });
        if channel.is_some() {
          return;
        }
        println!("private {}: '{}'", msg_data.user, msg_data.text);

        match self.process_message(cli, msg_data) {
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
    // TODO: react to original message?
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

  fn get_username(&mut self, cli: &slack::RtmClient, user_id: &str) -> Option<String> {
    cli
      .start_response()
      .users
      .as_ref()
      .and_then(|users| {
        users.iter().find(|user| match user.id {
          None => false,
          Some(ref id) => *id == user_id,
        })
      })
      .and_then(|ref u| u.real_name.clone())
  }

  fn process_message(&mut self, cli: &slack::RtmClient, msg: MsgData) -> Option<String> {
    // Process message
    let user_id = msg.user.as_str().clone();
    let username = self.get_username(cli, &user_id)?;
    match msg.text.as_str() {
      "done" => match self.daily_statuses.get_mut(user_id) {
        Some(status) => {
          let output = template_output(username, status.clone());
          self
            .daily_statuses
            .entry(msg.user)
            .and_modify(|e| *e = Vec::<String>::new());
          Some(output)
        }
        None => None,
      },
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
        let index = match usr_status.iter().position(|i| *i == msg.text) {
          Some(i) => i,
          None => return,
        };
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
    let _ = cli
      .sender()
      .send_message(&self.channel_id, message.as_str());
  }
}

fn template_output(user: String, status: Vec<String>) -> String {
  //TODO: get readable user name
  let mut output = format!("Status for {}:\n", user);
  output.extend(status.iter().map(|ref line| format!("  * {}\n", line)));
  output
}

fn main() {
  let args: Vec<String> = std::env::args().collect();
  if args.len() < 2 {
    panic!("Usage: cargo run -- <api-key> <channel ID>")
  }
  let api_key = &args[1];
  let channel_id = &args[2];
  let daily_statuses = HashMap::new();
  let mut handler = PMHandler {
    daily_statuses: daily_statuses,
    channel_id: channel_id.to_string(),
  };
  let r = slack::RtmClient::login_and_run(&api_key, &mut handler);
  match r {
    Ok(_) => {}
    Err(err) => panic!("Error: {}", err),
  }
}
