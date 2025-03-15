use core::f32;
use cosmic_text::Buffer;
use printpdf::*;
use std::{cmp::Ordering, fs};
use syntect::highlighting::ThemeSet;
use text_manipulation::TextWrapper;
mod helpers;
mod text_manipulation;
use ignore::{overrides::OverrideBuilder, WalkBuilder};
mod code_to_pdf;
use argh::FromArgs;
use code_to_pdf::{CodeToPdf, HighlighterConfig};
use std::time::Instant;

// This makes `FromArgs` happy
type StringVec = Vec<String>;
fn vec_from_string(s: &str) -> Result<StringVec, String> {
    Ok(s.split(",").into_iter().map(str::to_string).collect())
}
#[derive(FromArgs)]
/// Command line arguments
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
}
fn main() {
    let args: Arguments = argh::from_env();
    let path = args.walk_path;
    let page_dimensions: (f32, f32) = (210.0, 297.0);
    let mut doc = PdfDocument::new(&args.name);
    let helvetica_bytes = include_bytes!("../fonts/Helvetica.ttf");
    let font = ParsedFont::from_bytes(helvetica_bytes, 33, &mut vec![]).unwrap();
    let font_id = doc.add_font(&font);
    let ss = two_face::syntax::extra_newlines();
    let ts = ThemeSet::load_defaults();
    let walker = WalkBuilder::new(path.clone())
        .overrides({
            let mut builder = OverrideBuilder::new(path);
            for exclusion in args.exclude {
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
    let mut font_system = TextWrapper::font_bytes_to_font_system(helvetica_bytes);
    let mut c2pdf = CodeToPdf::new(
        font_id,
        page_dimensions,
        TextWrapper::new(
            Buffer::new(&mut font_system, cosmic_text::Metrics::new(14.0, 20.0)),
            font_system,
            cosmic_text::Wrap::Word,
        ),
    );
    let highlighter_config = HighlighterConfig::new(ss, ts);
    let start = Instant::now();
    c2pdf.process_files(walker, highlighter_config);
    let pages = c2pdf.get_pages();
    let num_pages = pages.len();
    let pdf_bytes: Vec<u8> = doc
        .with_pages(pages)
        .save(&PdfSaveOptions::default(), &mut vec![]);
    fs::write(args.out, pdf_bytes).unwrap();
    println!("Done!");
    println!(
        "Generated {} pages in {} seconds",
        num_pages,
        start.elapsed().as_secs_f32()
    )
}
