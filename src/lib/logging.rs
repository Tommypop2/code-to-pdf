//! Module for logging on a separate thread
use std::{
  sync::{
    Arc, Mutex,
    mpsc::{Receiver, Sender},
  },
  thread::{self, JoinHandle},
};

pub enum LoggerMessage {
  Message(String),
  Abort,
}
#[derive(Clone)]
pub struct Logger {
  tx: Sender<LoggerMessage>,
  handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl Logger {
  pub fn new(channel: (Sender<LoggerMessage>, Receiver<LoggerMessage>)) -> Logger {
    let (tx, rx) = channel;
    let handle = thread::spawn(move || {
      loop {
        let msg = if let Ok(msg) = rx.recv() {
          match msg {
            LoggerMessage::Message(msg) => msg,
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
  pub fn log(&self, item: String) {
    self.tx.send(LoggerMessage::Message(item)).unwrap()
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
  use std::sync::mpsc::channel;
  #[test]
  fn it_works() {
    let logger = Logger::new(channel());
    logger.log("Hello World!!!".into());
    logger.finish();
    assert!(false);
  }
}
