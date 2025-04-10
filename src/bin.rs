use argh::FromArgs;
use c2pdf::code_to_pdf::{CodeToPdf, HighlighterConfig};
use c2pdf::dimensions::Dimensions;
use c2pdf::font_loader::load_font;
use c2pdf::text_manipulation::TextWrapper;
use core::f32;
use ignore::{WalkBuilder, overrides::OverrideBuilder};
use printpdf::*;
use std::time::Instant;
use std::{cmp::Ordering, fs::File};

// This makes `FromArgs` happy
type StringVec = Vec<String>;
fn vec_from_string(s: &str) -> Result<StringVec, String> {
    Ok(s.split(",").map(str::to_string).collect())
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
    let ss = two_face::syntax::extra_newlines();
    let ts = two_face::theme::extra();
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
    let mut c2pdf = CodeToPdf::new(
        doc,
        font_id,
        page_dimensions,
        TextWrapper::new(font_bytes, args.font_size),
        args.page_text,
    );
    let highlighter_config = HighlighterConfig::new(
        ss,
        ts.get(two_face::theme::EmbeddedThemeName::InspiredGithub)
            .clone(),
    );
    let start = Instant::now();
    c2pdf.process_files(walker, highlighter_config);
    let processed_file_count = c2pdf.processed_file_count();
    let doc = c2pdf.document();
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
