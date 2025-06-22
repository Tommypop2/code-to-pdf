//! Module for logging on a separate thread
use std::{
  sync::{Arc, Mutex},
  thread::{self, JoinHandle},
};

/// Message for the logger
///
/// This includes the [`LoggerMessage::Complete`] and [`LoggerMessage::Abort`] signals, as well as [`LoggerMessage::Message`], which contains a message to log
pub enum LoggerMessage {
  /// Message to log
  Message(String),
  /// Complete signal
  Complete,
  /// Abort signal
  Abort,
}

/// Main logger struct
///
/// Allows for logging to stdout from a separate thread
#[derive(Clone)]
pub struct Logger {
  tx: crossbeam_channel::Sender<LoggerMessage>,
  handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl Logger {
  /// Creates a new [`Logger`]
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
  /// Creates a new logger without creating a logging thread
  pub fn new_without_logging_thread(sender: crossbeam_channel::Sender<LoggerMessage>) -> Logger {
    Logger {
      tx: sender,
      handle: Arc::new(Mutex::new(None)),
    }
  }
  /// Logs a string
  pub fn log_message(&self, item: String) {
    self.send_raw_message(LoggerMessage::Message(item));
  }
  /// Sends a raw message to the logger thread
  pub fn send_raw_message(&self, msg: LoggerMessage) {
    self.tx.send(msg).unwrap()
  }
  /// Waits for all threads to finish
  pub fn finish(self) -> std::thread::Result<()> {
    // Logger thread should exit once it processes this signal
    self.tx.send(LoggerMessage::Abort).unwrap();
    let mut lock = self.handle.lock().unwrap();
    let handle = lock.take().unwrap();
    handle.join()
  }
}
impl log::Log for Logger {
  fn enabled(&self, metadata: &log::Metadata) -> bool {
    if metadata.target() == "c2pdf" {
      metadata.level() <= log::Level::Trace
    } else {
      false
    }
  }
  fn log(&self, record: &log::Record) {
    if self.enabled(record.metadata()) {
      self.log_message(record.args().to_string());
    }
  }

  fn flush(&self) {}
}
#[cfg(test)]
mod tests {
  use log::info;

  use super::*;
  #[test]
  fn it_works() {
    let logger = Logger::new(crossbeam_channel::unbounded());
    info!("Hello world!!");
    _ = logger.finish();
  }
}
