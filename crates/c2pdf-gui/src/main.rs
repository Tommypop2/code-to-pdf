use std::{
  sync::{Arc, Mutex, atomic::AtomicI32, mpsc},
  thread::{self, sleep},
  time::Duration,
};

use floem::{
  action::open_file,
  ext_event::create_signal_from_channel,
  prelude::*,
  reactive::{Scope, SignalRead, create_effect},
};
use tokio::runtime::Runtime;

fn main() {
  let runtime = Runtime::new().expect("Couldn't start runtime");

  // We must make it so that the main task is under the tokio runtime so that APIs like
  // tokio::spawn work
  runtime.block_on(async { tokio::task::block_in_place(|| floem::launch(app_view)) })
}

fn app_view() -> impl IntoView {
  let w = floem::file::FileDialogOptions::new().select_directories();
  let cx = Scope::current();
  let (read, write) = cx.create_signal(2);
  let (tx, rx): (
    crossbeam_channel::Sender<i32>,
    crossbeam_channel::Receiver<i32>,
  ) = crossbeam_channel::unbounded();
  let status = create_signal_from_channel(rx);
  let num = Arc::new(Mutex::new(0));
  let thread_num = num.clone();
  let thread_tx = tx.clone();
  let increment_tx = tx.clone();
  let t = thread::spawn(move || {
    loop {
      sleep(Duration::from_millis(1000));
      dbg!("Incrementing");
      let x = thread_num
        .lock()
        .and_then(|mut x| {
          *x += 1;
          Ok(*x)
        })
        .unwrap();
      thread_tx.send(x).unwrap();
    }
  });
  create_effect(move |s| {
    println!("{}", read.get());
  });
  let increment_num = num.clone();
  let decrement_num = num.clone();
  v_stack((
    button("Open File Picker").action(move || {
      open_file(w.clone(), |f| {
        dbg!(f);
      })
    }),
    h_stack((
      button("Increment").action(move || {
        let x = increment_num
          .clone()
          .lock()
          .and_then(|mut x| {
            *x += 1;
            Ok(*x)
          })
          .unwrap();
        tx.send(x);
      }),
      label(move || format!("Value: {}", status.get().unwrap_or(0))),
      button("Decrement").action(move || {
        let x = decrement_num
          .clone()
          .lock()
          .and_then(|mut x| {
            *x -= 1;
            Ok(*x)
          })
          .unwrap();

        increment_tx.send(x);
      }),
    ))
    .style(|s| s.size_full().items_center().justify_center().gap(10)),
  ))
  .style(|s| s.size_full().items_center().justify_center().gap(10))
}
