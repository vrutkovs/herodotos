use env_logger;
use log::{info, trace};
use slack;
use slack_api;
use std::collections::HashMap;

struct MsgData {
  user: String,
  text: String,
}

struct PMHandler {
  status_channel_id: String,
  daily_statuses: HashMap<String, Vec<String>>,
}

impl slack::EventHandler for PMHandler {
  fn on_event(&mut self, cli: &slack::RtmClient, event: slack::Event) {
    trace!("on_event(event: {:?})", event);
    let _ = self.process_event(cli, event);
    // TODO: react to original message?
  }

  fn on_close(&mut self, _: &slack::RtmClient) {
    trace!("on_close");
    //TODO: set offline signal
  }

  fn on_connect(&mut self, _: &slack::RtmClient) {
    trace!("on_connect");
    //TODO: set online signal
  }
}

impl PMHandler {
  pub fn new(channel_id: &str) -> PMHandler {
    PMHandler {
      status_channel_id: channel_id.to_string(),
      daily_statuses: HashMap::new(),
    }
  }

  fn process_event(&mut self, cli: &slack::RtmClient, event: slack::Event) -> Option<()> {
    trace!("processing event {:?}", event);
    let slack_message = match event {
      slack::Event::Message(m) => Box::leak(m),
      _ => return None,
    };
    match &slack_message {
      slack::Message::Standard(m) => {
        trace!("standard message found");
        let msg_data = &get_message(m)?;
        let msg_channel_id = String::from(m.channel.as_ref()?);
        if !is_private_message(cli, msg_channel_id.clone()) {
          trace!("message for channel {}, not PM", msg_channel_id.clone());
          return None;
        }
        info!(
          "Processing message from {}: '{}'",
          msg_data.user, msg_data.text
        );

        let msg = self.process_message(cli, msg_data, &msg_channel_id)?;

        trace!("got msg: '{}'", msg);

        Some(send_message(cli, self.status_channel_id.clone(), msg))
      }
      slack::Message::MessageDeleted(m) => {
        trace!("processing deleted message");
        let msg = &get_deleted_message(m)?;
        Some(self.process_deleted_message(msg))
      }
      slack::Message::MessageChanged(m) => {
        trace!("processing updated message");
        let msg = &get_edited_message(m)?;
        Some(self.process_edited_message(msg))
      }
      _ => None,
    }
  }

  fn process_message(
    &mut self,
    cli: &slack::RtmClient,
    msg: &MsgData,
    msg_channel_id: &str,
  ) -> Option<String> {
    // Process message
    let user_id = msg.user.as_str().clone();
    let username = get_username(cli, &user_id)?;
    let user_msgs = self
      .daily_statuses
      .entry(msg.user.clone())
      .or_insert(Vec::<String>::new());

    trace!("got user_msgs: '{:?}'", user_msgs);

    let result = match msg.text.as_str() {
      // Return the message to post
      "done" => {
        // Return status message, reset user_msgs
        let output = template_output(username, user_msgs.to_vec());
        user_msgs.clear();
        Some(output)
      }
      "preview" => {
        // Preview posts message back to user, not public channel
        let output = template_output(username, user_msgs.to_vec());
        send_message(cli, msg_channel_id.to_string(), output);
        None
      }
      _ => {
        // Store messages
        user_msgs.push(msg.text.to_string());
        trace!("new user_msgs: '{:?}'", user_msgs);
        None
      }
    };
    trace!("new daily_statuses: '{:?}'", self.daily_statuses);
    result
  }

  fn process_deleted_message(&mut self, msg: &MsgData) {
    self
      .daily_statuses
      .get_mut(&msg.user)
      .and_then(|usr_status| {
        let index = usr_status.iter().position(|i| *i == msg.text)?;
        Some(usr_status.remove(index))
      })
      .unwrap();
  }

  fn process_edited_message(&mut self, msgs: &(MsgData, MsgData)) {
    let previous_message = &msgs.0;
    let new_message = &msgs.1;

    self
      .daily_statuses
      .get_mut(&previous_message.user)
      .and_then(|usr_status| {
        let index = usr_status
          .iter()
          .position(|i| *i == previous_message.text)
          .unwrap();
        Some(std::mem::replace(
          &mut usr_status[index],
          new_message.text.to_string(),
        ))
      })
      .unwrap();
  }
}

fn send_message(cli: &slack::RtmClient, channel_id: String, message: String) {
  info!("Sending message to channel {}: '{:?}'", channel_id, message);
  let _ = cli
    .sender()
    .send_message(channel_id.as_str(), message.as_str());
}

fn is_private_message(cli: &slack::RtmClient, channel_id: String) -> bool {
  let channel = cli.start_response().channels.as_ref().and_then(|channels| {
    channels.iter().find(|chan| match chan.id {
      None => false,
      Some(ref id) => *id == channel_id,
    })
  });
  channel.is_none()
}

fn get_username(cli: &slack::RtmClient, user_id: &str) -> Option<String> {
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

fn get_message(m: &slack_api::MessageStandard) -> Option<MsgData> {
  let text = m.text.as_ref()?;
  let user = m.user.as_ref()?;
  Some(MsgData {
    text: text.to_string(),
    user: user.to_string(),
  })
}

fn get_deleted_message(m: &slack_api::MessageMessageDeleted) -> Option<MsgData> {
  let previous_message = m.previous_message.as_ref()?;
  //TODO: Merge with get_message?
  let text = previous_message.text.as_ref()?;
  let user = previous_message.user.as_ref()?;
  Some(MsgData {
    text: text.to_string(),
    user: user.to_string(),
  })
}

fn template_output(user: String, status: Vec<String>) -> String {
  //TODO: get readable user name
  let mut output = format!("Status for {}:\n", user);
  output.extend(status.iter().map(|ref line| format!("  * {}\n", line)));
  output
}

fn get_edited_message(m: &slack_api::MessageMessageChanged) -> Option<(MsgData, MsgData)> {
  let previous_message = m.previous_message.as_ref()?;
  let new_message = m.message.as_ref()?;

  let previous_text = previous_message.text.as_ref()?;
  let user = previous_message.user.as_ref()?;
  let new_text = new_message.text.as_ref()?;
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

fn main() {
  let args: Vec<String> = std::env::args().collect();
  if args.len() < 2 {
    panic!("Usage: cargo run -- <api-key> <channel ID>")
  }

  env_logger::init();

  let api_key = &args[1];
  let channel_id = &args[2];
  let mut handler = PMHandler::new(channel_id);
  trace!("Starting slack bot");
  loop {
    let r = slack::RtmClient::login_and_run(&api_key, &mut handler);
    match r {
      Ok(_) => {}
      Err(err) => println!("Error: {}", err),
    }
  }
}
