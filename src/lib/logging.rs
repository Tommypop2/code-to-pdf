use std::{
  sync::mpsc::{Receiver, Sender, channel},
  thread,
};

struct Logger {}

impl Logger {}

pub fn create_logger_thread<'a>(rx: Receiver<String>) {
  thread::spawn(move || {
    loop {
      let msg = if let Ok(msg) = rx.try_recv() {
        msg
      } else {
        continue;
      };
      dbg!(msg);
    }
  });
}
