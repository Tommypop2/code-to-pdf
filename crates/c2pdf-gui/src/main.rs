use std::{
  cell::RefCell,
  fs::File,
  path::PathBuf,
  sync::{Arc, Mutex},
  thread::{self},
  time::Instant,
};

use c2pdf::{
  ParsedFont, PdfDocument, PdfSaveOptions,
  code_to_pdf::CodeToPdf,
  dimensions::Dimensions,
  font_loader,
  logging::{Logger, LoggerMessage},
};
use floem::{
  action::open_file,
  ext_event::create_signal_from_channel,
  prelude::*,
  reactive::{SignalRead, create_effect},
};
#[derive(Clone)]
enum JobStatus {
  // No job currently happening
  Stopped,
  // Thread is running
  Running,
  // Done, need to write result
  Complete,
}
fn main() {
  floem::launch(app_view);
}

fn app_view() -> impl IntoView {
  let w = floem::file::FileDialogOptions::new().select_directories();
  let (dir_path, set_dir_path) = create_signal::<Option<PathBuf>>(None);
  let (job_status, set_job_status) = create_signal(JobStatus::Stopped);
  let (tx, rx) = crossbeam_channel::unbounded();
  let logger = Logger::new_without_logging_thread(tx);

  let logger_message: floem::reactive::ReadSignal<Option<LoggerMessage>> =
    create_signal_from_channel(rx.clone());

  let thread_handle: Arc<Mutex<Option<thread::JoinHandle<usize>>>> = Arc::new(Mutex::new(None));
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
  create_effect(move |_| {
    let binding = logger_message.read();
    let message = &*binding.borrow();
    let message = if let Some(m) = message { m } else { return };
    if let LoggerMessage::Complete = message {
      set_job_status.set(JobStatus::Complete);
    }
  });
  let start_time = RefCell::new(Instant::now());
  create_effect(move |_| match job_status.get() {
    JobStatus::Complete => {
      // Task is complete so can join thread
      let result = thread_handle2.lock().unwrap().take().unwrap();
      let number_files_processed = result.join().unwrap();
      let time_taken = start_time.borrow().elapsed();
      println!(
        "Done!! Processed {number_files_processed} files in {:.2}s",
        time_taken.as_secs_f32()
      );
      set_job_status.set(JobStatus::Stopped);
    }
    JobStatus::Stopped => {
      // Do nothing in the stopped state
    }
    JobStatus::Running => {
      // Start job timer
      *start_time.borrow_mut() = Instant::now();
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
      let logger_for_thread = logger.clone();
      let path_for_thread = path.clone();
      set_job_status.set(JobStatus::Running);
      thread_handle
        .lock()
        .unwrap()
        .replace(std::thread::spawn(move || {
          let mut doc = PdfDocument::new("doc");
          let font = ParsedFont::from_bytes(&bytes, 0, &mut vec![]).unwrap();
          let font_id = doc.add_font(&font);
          let (doc_subset, number_files_processed) = CodeToPdf::run_parallel(
            font_id,
            &bytes,
            path_for_thread.clone(),
            vec![],
            Dimensions::default(),
            12.0,
            None,
            &logger_for_thread,
            None,
          );
          doc_subset.lock().unwrap().to_document(&mut doc);
          let f = File::create(path_for_thread.join("output.pdf")).unwrap();
          let mut f = std::io::BufWriter::new(f);
          doc.save_writer(&mut f, &PdfSaveOptions::default(), &mut vec![]);
          logger_for_thread.send_raw_message(LoggerMessage::Complete);
          number_files_processed
        }));
      // wasd.join();
    }),
    label(move || {
      let binding = logger_message.read();
      let message = &*binding.borrow();
      match message {
        Some(LoggerMessage::Message(s)) => s.to_string(),
        Some(LoggerMessage::Complete) => "Done!!".into(),
        _ => "".into(),
      }
    }),
  ))
  .style(|s| s.size_full().items_center().justify_center().gap(10))
}
