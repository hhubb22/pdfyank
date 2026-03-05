use arboard::{Clipboard, ImageData};
use clap::Parser;
use image::save_buffer;
use pdfium_rs::{Document, Library};

#[derive(Parser)]
#[command(about = "Render a PDF page to a PNG image")]
struct Args {
    /// Path to the PDF file
    pdf: String,

    /// Zero-based page index
    #[arg(short, long, default_value_t = 0)]
    page: u32,

    /// Rendering DPI (higher = better quality)
    #[arg(short, long, default_value_t = 300.0)]
    dpi: f32,

    /// Output PNG path (if omitted, only copy to clipboard)
    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let _lib = Library::init();
    let doc = Document::open(&args.pdf)?;
    let page = doc.page(args.page)?;
    let result = page.render(args.dpi)?;

    // RenderResult has stride which may include padding; extract rows without padding
    let row_bytes = result.width * 4;
    let data = if result.stride == row_bytes {
        result.data
    } else {
        let mut packed = Vec::with_capacity((row_bytes * result.height) as usize);
        for y in 0..result.height {
            let start = (y * result.stride) as usize;
            let end = start + row_bytes as usize;
            packed.extend_from_slice(&result.data[start..end]);
        }
        packed
    };

    if let Some(ref output) = args.output {
        save_buffer(
            output,
            &data,
            result.width,
            result.height,
            image::ColorType::Rgba8,
        )?;
    }

    let mut clipboard = Clipboard::new()?;
    clipboard.set_image(ImageData {
        width: result.width as usize,
        height: result.height as usize,
        bytes: data.into(),
    })?;

    match &args.output {
        Some(output) => eprintln!(
            "Rendered page {} at {} DPI → {} ({}×{} px, copied to clipboard)",
            args.page, args.dpi, output, result.width, result.height
        ),
        None => eprintln!(
            "Rendered page {} at {} DPI ({}×{} px, copied to clipboard)",
            args.page, args.dpi, result.width, result.height
        ),
    }

    Ok(())
}
