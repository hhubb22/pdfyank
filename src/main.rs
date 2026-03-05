use anyhow::{bail, Context, Result};
use arboard::{Clipboard, ImageData};
use clap::Parser;
use image::save_buffer;
use pdfium_rs::{Document, Library};

const MAX_PIXELS: usize = 200_000_000;

#[derive(Parser)]
#[command(about = "Render a PDF page to a PNG image", version)]
struct Args {
    /// Path to the PDF file
    pdf: String,

    /// 1-based page number
    #[arg(short, long, default_value_t = 1)]
    page: u32,

    /// Rendering DPI (higher = better quality)
    #[arg(short, long, default_value_t = 300.0)]
    dpi: f32,

    /// Output PNG path (if omitted, copy to clipboard only)
    #[arg(short, long)]
    output: Option<String>,

    /// Copy to clipboard (default when --output is omitted)
    #[arg(short, long)]
    clipboard: bool,

    /// Suppress status messages
    #[arg(short, long)]
    quiet: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // --- Input validation ---
    if !args.dpi.is_finite() || args.dpi <= 0.0 {
        bail!("--dpi must be a finite positive number (got {})", args.dpi);
    }
    if args.dpi > 1200.0 {
        bail!("--dpi cannot exceed 1200 (got {})", args.dpi);
    }
    if args.page == 0 {
        bail!("--page is 1-based and must be at least 1");
    }

    let use_clipboard = args.clipboard || args.output.is_none();

    let _lib = Library::init();
    let doc =
        Document::open(&args.pdf).with_context(|| format!("failed to open PDF: {}", args.pdf))?;

    let page_count = doc.page_count();
    let page_index = args.page - 1;
    if page_index >= page_count {
        bail!(
            "page {} is out of range (document has {} page{})",
            args.page,
            page_count,
            if page_count == 1 { "" } else { "s" }
        );
    }

    let page = doc
        .page(page_index)
        .with_context(|| format!("failed to load page {}", args.page))?;
    let result = page
        .render(args.dpi)
        .with_context(|| format!("failed to render page {} at {} DPI", args.page, args.dpi))?;

    // --- Safe pixel-data packing with checked arithmetic ---
    let width = result.width as usize;
    let height = result.height as usize;
    let stride = result.stride as usize;
    let row_bytes = width.checked_mul(4).context("width too large")?;

    if stride < row_bytes {
        bail!(
            "invalid render result: stride ({stride}) < row_bytes ({row_bytes})"
        );
    }

    let total_pixels = width.checked_mul(height).context("image dimensions too large")?;
    if total_pixels > MAX_PIXELS {
        bail!(
            "refusing to process {total_pixels} pixels (limit is {MAX_PIXELS}); lower --dpi or use a smaller page"
        );
    }

    let expected_len = stride.checked_mul(height).context("image too large")?;
    if result.data.len() < expected_len {
        bail!(
            "invalid render result: buffer too small (have {}, need at least {expected_len})",
            result.data.len()
        );
    }

    let data = if stride == row_bytes {
        result.data
    } else {
        let packed_len = row_bytes.checked_mul(height).context("image too large")?;
        let mut packed = Vec::with_capacity(packed_len);
        for row in result.data[..expected_len].chunks_exact(stride) {
            packed.extend_from_slice(&row[..row_bytes]);
        }
        packed
    };

    // --- Output ---
    if let Some(ref output) = args.output {
        save_buffer(
            output,
            &data,
            result.width,
            result.height,
            image::ColorType::Rgba8,
        )
        .with_context(|| format!("failed to save image to {output}"))?;
    }

    if use_clipboard {
        let mut clipboard =
            Clipboard::new().context("failed to access clipboard (headless environment?)")?;
        clipboard
            .set_image(ImageData {
                width,
                height,
                bytes: data.into(),
            })
            .context("failed to copy image to clipboard")?;
    }

    if !args.quiet {
        let clipboard_msg = if use_clipboard {
            ", copied to clipboard"
        } else {
            ""
        };
        match &args.output {
            Some(output) => eprintln!(
                "Rendered page {} at {} DPI → {} ({}×{} px{})",
                args.page, args.dpi, output, result.width, result.height, clipboard_msg
            ),
            None => eprintln!(
                "Rendered page {} at {} DPI ({}×{} px{})",
                args.page, args.dpi, result.width, result.height, clipboard_msg
            ),
        }
    }

    Ok(())
}
