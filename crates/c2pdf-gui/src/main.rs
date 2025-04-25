use std::{
  fs::File,
  mem,
  path::PathBuf,
  sync::{
    Arc, Mutex,
    atomic::AtomicI32,
    mpsc::{self, channel},
  },
  thread::{self, sleep},
  time::Duration,
};

use c2pdf::{
  ParsedFont, PdfDocument, PdfSaveOptions,
  code_to_pdf::{CodeToPdf, DocumentSubset},
  dimensions::Dimensions,
  font_loader,
  logging::{Logger, LoggerMessage},
};
use floem::{
  action::open_file,
  ext_event::create_signal_from_channel,
  prelude::*,
  reactive::{Scope, SignalRead, SignalWrite, create_effect},
};
use tokio::runtime::Runtime;
#[derive(Clone)]
enum JobStatus {
  // No job currently happening
  STOPPED,
  // Thread is running
  RUNNING,
  // Done, need to write result
  COMPLETE,
}
fn main() {
  let runtime = Runtime::new().expect("Couldn't start runtime");

  // We must make it so that the main task is under the tokio runtime so that APIs like
  // tokio::spawn work
  runtime.block_on(async { tokio::task::block_in_place(|| floem::launch(app_view)) })
}

fn app_view() -> impl IntoView {
  let w = floem::file::FileDialogOptions::new().select_directories();
  let (dir_path, set_dir_path) = create_signal::<Option<PathBuf>>(None);
  let (job_status, set_job_status) = create_signal(JobStatus::STOPPED);
  let (tx, rx) = crossbeam_channel::unbounded();
  let logger = Logger::new_without_logging_thread(tx);

  let logger_message: floem::reactive::ReadSignal<Option<LoggerMessage>> =
    create_signal_from_channel(rx.clone());

  let thread_handle: Arc<Mutex<Option<thread::JoinHandle<(Arc<Mutex<DocumentSubset>>, usize)>>>> =
    Arc::new(Mutex::new(None));
  let thread_handle2 = thread_handle.clone();
  create_effect(move |_| {
    println!(
      "{}",
      dir_path
        .get()
        .and_then(|path| path.to_str().map(|x| x.to_string()))
        .unwrap_or("".to_string())
    );
  });
  let doc = Arc::new(Mutex::new(PdfDocument::new("asd")));
  let doc2 = doc.clone();
  create_effect(move |_| {
    let binding = logger_message.read();
    let message = &*binding.borrow();
    let message = if let Some(m) = message { m } else { return };
    match message {
      LoggerMessage::Complete => {
        set_job_status.set(JobStatus::COMPLETE);
      }
      _ => {}
    }
  });
  create_effect(move |_| match job_status.get() {
    JobStatus::COMPLETE => {
      // Task is complete so can write result
      let result = thread_handle2.lock().unwrap().take().unwrap();
      let (doc_subset, number_files_processed) = result.join().unwrap();
      // Write the document. This unfortunately has to be done on the main thread :(
      doc_subset
        .lock()
        .unwrap()
        .to_document(&mut doc2.lock().unwrap());
      let f = File::create(dir_path.get_untracked().unwrap().join("output.pdf")).unwrap();
      let mut f = std::io::BufWriter::new(f);
      println!("Saving...");
      doc2
        .lock()
        .unwrap()
        .save_writer(&mut f, &PdfSaveOptions::default(), &mut vec![]);
      println!("Saving complete...");
      // Replace document
      _ = mem::replace(&mut *doc2.lock().unwrap(), PdfDocument::new("asd"));

      set_job_status.set(JobStatus::STOPPED);
    }
    JobStatus::STOPPED => {
      println!("Done!!");
    }
    JobStatus::RUNNING => {
      // Do nothing here
    }
  });
  v_stack((
    // v_stack(()),
    button("Open File Picker").action(move || {
      open_file(w.clone(), move |f| {
        if let Some(file) = f {
          let path = file.path[0].clone();
          set_dir_path.set(Some(path));
        };
      });
    }),
    label(move || {
      let path = dir_path
        .get()
        .and_then(|path| path.to_str().map(|x| x.to_string()));
      if let Some(p) = path {
        format!("Path: {}", p)
      } else {
        "No path selected".into()
      }
    }),
    button("Process!").action(move || {
      // Here we need to actually invoke `code-to-pdf`
      let (bytes, _) = font_loader::load_font(Some("CaskaydiaCove Nerd Font Mono".into()));
      let path = if let Some(p) = dir_path.get() {
        p
      } else {
        return;
      };
      let font = ParsedFont::from_bytes(&bytes, 0, &mut vec![]).unwrap();
      let font_id = doc.lock().unwrap().add_font(&font);
      let logger_for_thread = logger.clone();
      let path_for_thread = path.clone();
      set_job_status.set(JobStatus::RUNNING);
      thread_handle
        .lock()
        .unwrap()
        .replace(std::thread::spawn(move || {
          let res = CodeToPdf::run_parallel(
            font_id,
            &bytes,
            path_for_thread,
            vec![],
            Dimensions::default(),
            12.0,
            None,
            &logger_for_thread,
          );
          logger_for_thread.send_raw_message(LoggerMessage::Complete);
          res
        }));
      // wasd.join();
    }),
    label(move || {
      let binding = logger_message.read();
      let message = &*binding.borrow();
      match message {
        Some(LoggerMessage::Message(s)) => format!("{}", s),
        Some(LoggerMessage::Complete) => "Done!!".into(),
        _ => "".into(),
      }
    }),
  ))
  .style(|s| s.size_full().items_center().justify_center().gap(10))
}
