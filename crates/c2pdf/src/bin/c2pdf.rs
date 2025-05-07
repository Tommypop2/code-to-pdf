use argh::FromArgs;
use c2pdf::code_to_pdf::CodeToPdf;
use c2pdf::dimensions::Dimensions;
use c2pdf::font_loader::load_font;
use c2pdf::logging::Logger;
use core::f32;
use printpdf::*;
use std::fs::File;
use std::num::NonZeroU8;
use std::path::PathBuf;
use std::time::Instant;
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

  /// number of threads to use for processing
  #[argh(option)]
  threads: Option<NonZeroU8>,
}
fn main() {
  // Parse args
  let args: Arguments = argh::from_env();
  // Set up logger
  let logger = Logger::new(crossbeam_channel::unbounded());
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
  let start = Instant::now();
  let (doc_subset, processed_file_count) = CodeToPdf::run_parallel(
    font_id,
    font_bytes,
    PathBuf::from(path),
    args.exclude,
    page_dimensions,
    args.font_size,
    args.page_text,
    &logger,
    args.threads,
  );
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
