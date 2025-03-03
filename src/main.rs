//! Prints highlighted HTML for a file to stdout.
//! Basically just wraps a body around `highlighted_html_for_file`
use printpdf::html;
use printpdf::*;
use std::collections::BTreeMap;
use std::fs;
use syntect::highlighting::{Color, ThemeSet};
use syntect::html::highlighted_html_for_file;
use syntect::parsing::SyntaxSet;
fn main() {
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Please pass in a file to highlight");
        return;
    }

    let style = "
        pre {
            font-size:13px;
            font-family: Consolas, \"Liberation Mono\", Menlo, Courier, monospace;
        }";
    println!("<head><title>{}</title></head>", &args[1]);
    let theme = &ts.themes["base16-ocean.dark"];
    let c = theme.settings.background.unwrap_or(Color::WHITE);
    println!(
        "<body style=\"background-color:#{:02x}{:02x}{:02x};\">\n",
        c.r, c.g, c.b
    );
    let html = highlighted_html_for_file(&args[1], &ss, theme)
        .unwrap()
        .replace("<pre style=\"background-color:#2b303b;\">", "")
        .replace("</pre>", "")
        .replace("span", "div");
    println!("{}", html);
    println!("</body>");
    let options = XmlRenderOptions {
        // named images to be used in the HTML, i.e. ["image1.png" => DecodedImage(image1_bytes)]
        images: BTreeMap::new(),
        // named fonts to be used in the HTML, i.e. ["Roboto" => DecodedImage(roboto_bytes)]
        fonts: BTreeMap::new(),
        // default page width, printpdf will auto-page-break
        page_width: Mm(210.0),
        // default page height
        page_height: Mm(297.0),
        components: vec![],
    };
    let mut warnings = vec![];
    let pages = PdfDocument::new("Yes")
        .html_to_pages(&html, options, &mut warnings)
        .unwrap();
    let pdf_bytes = PdfDocument::new("My PDF")
        .with_pages(pages)
        .save(&PdfSaveOptions::default(), &mut warnings);
    // let pdf_bytes = PdfDocument::new("My PDF")
    //     .with_html(html, &options)
    //     .unwrap()
    //     .save(&PdfSaveOptions::default());
    fs::write("./hello.pdf", pdf_bytes);
}
