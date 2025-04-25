#![warn(missing_docs)]

//! # Code To PDF
//!
//! This crate provides primitives for generating PDFs containing syntax-highlighted code
//!
//! [`code_to_pdf::CodeToPdf`] is the main struct for this so is likely the best place to start

use std::{
  cmp::Ordering,
  path::PathBuf,
  sync::{Arc, Mutex},
};

use code_to_pdf::{CodeToPdf, DocumentSubset, HighlighterConfig};
use dimensions::Dimensions;
use ignore::{WalkBuilder, overrides::OverrideBuilder};
use logging::Logger;
use printpdf::FontId;
use rayon::iter::{ParallelBridge, ParallelIterator};
use text_manipulation::TextWrapper;
use thread_local::ThreadLocal;

pub mod code_to_pdf;
pub mod dimensions;
pub mod font_loader;
pub mod helpers;
pub mod logging;
pub mod text_manipulation;

pub use printpdf::{ParsedFont, PdfDocument, PdfSaveOptions};

// Do this here, until I find a good name for a module to plate it in :)
// Maybe `easy`, like what `syntect` has
impl CodeToPdf {
  /// Helper function that handles everything for the basic use-case
  pub fn run_parallel(
    font_id: FontId,
    font_bytes: &[u8],
    path: PathBuf,
    exclusions: Vec<String>,
    page_dimensions: Dimensions,
    font_size: f32,
    page_text: Option<String>,
    logger: &Logger,
  ) -> (Arc<Mutex<DocumentSubset>>, usize) {
    let doc_subset = DocumentSubset::default();
    let ss = two_face::syntax::extra_newlines();
    let ts = two_face::theme::extra();
    let walker = WalkBuilder::new(path.clone())
      .overrides({
        let mut builder = OverrideBuilder::new(path);
        for exclusion in exclusions.clone() {
          builder.add(&("!".to_string() + &exclusion)).unwrap();
        }
        builder.build().unwrap()
      })
      // Ensure that files are given higher precidence than folders
      // (want files in a folder to be printed breadth-first)
      .sort_by_file_path(|x, y| {
        {
          if x.is_dir() && !y.is_dir() {
            Ordering::Less
          } else if y.is_dir() && !x.is_dir() {
            Ordering::Greater
          } else {
            Ordering::Equal
          }
        }
        .reverse()
      })
      .build();

    let local_c2pdf = ThreadLocal::<Arc<Mutex<CodeToPdf>>>::new();
    let local_highlighter_config = ThreadLocal::<Arc<Mutex<HighlighterConfig>>>::new();

    let doc_subset = Arc::new(Mutex::new(doc_subset));

    walker.enumerate().par_bridge().for_each(|(i, result)| {
      // let mut doc = PdfDocument::new(&args.name);
      let c2pdf_mutex = local_c2pdf.get_or(|| {
        Arc::new(Mutex::new(CodeToPdf::new(
          doc_subset.clone(),
          font_id.clone(),
          page_dimensions.clone(),
          TextWrapper::new(font_bytes, font_size),
          page_text.clone(),
        )))
      });
      let highlight_config_mutex = local_highlighter_config.get_or(|| {
        Arc::new(Mutex::new(HighlighterConfig::new(
          ss.clone(),
          ts.get(two_face::theme::EmbeddedThemeName::InspiredGithub)
            .clone(),
        )))
      });
      match result {
        Ok(entry) => {
          if entry.file_type().is_some_and(|f| f.is_file()) {
            let path = entry.path();
            logger.log(format!(
              "Generating pages for {}, index {i}",
              path.display()
            ));
            if let Err(err) = c2pdf_mutex.lock().unwrap().process_file(
              path,
              &highlight_config_mutex.lock().unwrap(),
              i,
            ) {
              logger.log(format!("ERROR: {}", err));
            }
          }
        }
        Err(err) => {
          println!("ERROR: {}", err);
        }
      }
    });
    let mut processed_file_count = 0;
    for local in local_c2pdf.iter() {
      processed_file_count += local.lock().unwrap().processed_file_count();
    }

    // doc_subset.lock().unwrap().to_document(doc);
    (doc_subset, processed_file_count)
  }
}
