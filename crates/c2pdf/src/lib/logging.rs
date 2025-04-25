//! Module for logging on a separate thread
use std::{
  sync::{
    Arc, Mutex,
  },
  thread::{self, JoinHandle},
};
pub enum LoggerMessage {
  Message(String),
  Complete,
  Abort,
}
#[derive(Clone)]
pub struct Logger {
  tx: crossbeam_channel::Sender<LoggerMessage>,
  handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl Logger {
  pub fn new(
    channel: (
      crossbeam_channel::Sender<LoggerMessage>,
      crossbeam_channel::Receiver<LoggerMessage>,
    ),
  ) -> Logger {
    let (tx, rx) = channel;
    let handle = thread::spawn(move || {
      loop {
        let msg = if let Ok(msg) = rx.recv() {
          match msg {
            LoggerMessage::Message(msg) => msg,
            LoggerMessage::Complete => continue,
            LoggerMessage::Abort => break,
          }
        } else {
          continue;
        };
        println!("{}", msg);
      }
    });
    Logger {
      tx,
      handle: Arc::new(Mutex::new(Some(handle))),
    }
  }
  pub fn new_without_logging_thread(sender: crossbeam_channel::Sender<LoggerMessage>) -> Logger {
    Logger {
      tx: sender,
      handle: Arc::new(Mutex::new(None)),
    }
  }
  pub fn log(&self, item: String) {
    self.tx.send(LoggerMessage::Message(item)).unwrap()
  }
  pub fn send_raw_message(&self, msg: LoggerMessage) {
    self.tx.send(msg).unwrap()
  }
  /// Waits for all threads to finish
  pub fn finish(self) {
    // Logger thread should exit once it processes this signal
    self.tx.send(LoggerMessage::Abort).unwrap();
    let mut lock = self.handle.lock().unwrap();
    let handle = lock.take().unwrap();
    handle.join();
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn it_works() {
    let logger = Logger::new(crossbeam_channel::unbounded());
    logger.log("Hello World!!!".into());
    logger.finish();
    assert!(false);
  }
}
