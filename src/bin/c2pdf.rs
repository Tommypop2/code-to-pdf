use argh::FromArgs;
use c2pdf::code_to_pdf::{CodeToPdf, DocumentSubset, HighlighterConfig};
use c2pdf::dimensions::Dimensions;
use c2pdf::font_loader::load_font;
use c2pdf::text_manipulation::TextWrapper;
use core::f32;
use ignore::{WalkBuilder, overrides::OverrideBuilder};
use printpdf::*;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::{cmp::Ordering, fs::File};
use thread_local::ThreadLocal;
// This makes `FromArgs` happy
type StringVec = Vec<String>;
fn vec_from_string(s: &str) -> Result<StringVec, String> {
  Ok(s.split(",").map(str::to_string).collect())
}
#[derive(FromArgs)]
/// Generates a PDF from your source code
struct Arguments {
  /// the path to walk for files to highlight
  #[argh(positional)]
  walk_path: String,

  /// path to output PDF to
  #[argh(option, default = "String::from(\"output.pdf\")")]
  out: String,
  /// comma separated string of globs to exclude.
  /// Default exclusions are `pnpm-lock.yaml` and `Cargo.lock`
  #[argh(
    option,
    from_str_fn(vec_from_string),
    default = "vec![\"pnpm-lock.yaml\".into(), \"Cargo.lock\".into()]"
  )]
  exclude: StringVec,

  /// name of PDF
  #[argh(option, default = "String::from(\"Project Code\")")]
  name: String,

  /// name (will load from system fonts) or path of font to use
  ///
  /// code-to-pdf will use the bundled `Helvetica` font by default, or if the font provided cannot be loaded
  #[argh(option)]
  font: Option<String>,

  /// size of the font in the PDF in point
  #[argh(option, default = "12.0")]
  font_size: f32,

  /// size of the top margin (20.0 by default)
  #[argh(option, default = "20.0")]
  margin_top: f32,

  /// size of the bottom margin (5.0 by default)
  #[argh(option, default = "5.0")]
  margin_bottom: f32,

  /// size of the left margin (10.0 by default)
  #[argh(option, default = "10.0")]
  margin_left: f32,

  /// size of the right margin (10.0 by default)
  #[argh(option, default = "10.0")]
  margin_right: f32,

  /// text to add to (the top of) every page
  #[argh(option)]
  page_text: Option<String>,
}
fn main() {
  let args: Arguments = argh::from_env();
  let path = args.walk_path;
  let page_dimensions = Dimensions::new(
    Mm(210.0),
    Mm(297.0),
    Mm(args.margin_top),
    Mm(args.margin_bottom),
    Mm(args.margin_left),
    Mm(args.margin_right),
  );
  let mut doc = PdfDocument::new(&args.name);
  let (font_bytes, used_bundled) = load_font(args.font);
  if used_bundled {
    eprintln!("Unable to load provided font")
  }
  let font_bytes = &*font_bytes;
  let font = ParsedFont::from_bytes(font_bytes, 0, &mut vec![]).unwrap();
  let font_id = doc.add_font(&font);
  let doc_subset = DocumentSubset::default();
  let ss = two_face::syntax::extra_newlines();
  let ts = two_face::theme::extra();
  let walker = WalkBuilder::new(path.clone())
    .overrides({
      let mut builder = OverrideBuilder::new(path);
      for exclusion in args.exclude.clone() {
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
  let start = Instant::now();
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
        TextWrapper::new(font_bytes, args.font_size),
        args.page_text.clone(),
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
          if let Err(err) = c2pdf_mutex.lock().unwrap().process_file(
            entry.path(),
            &highlight_config_mutex.lock().unwrap(),
            i,
          ) {
            println!("ERROR: {}", err);
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

  doc_subset.lock().unwrap().to_document(&mut doc);
  let num_pages = doc.pages.len();
  // let before_write = Instant::now();
  let f = File::create(args.out).unwrap();
  let mut f = std::io::BufWriter::new(f);
  doc.save_writer(&mut f, &PdfSaveOptions::default(), &mut vec![]);
  // println!("Written in {}", before_write.elapsed().as_micros());
  println!("Done!");
  println!(
    "Processed {} files and generated {} pages in {} seconds",
    processed_file_count,
    num_pages,
    start.elapsed().as_secs_f32()
  )
}
