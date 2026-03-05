# pdfyank

Render a PDF page to a PNG image or copy it directly to the clipboard.

## Install

```bash
cargo install --git https://github.com/hhubb22/pdfyank
```

> On first build, `libpdfium` is downloaded automatically and installed to `~/.local/lib/`.

## Usage

```bash
# Copy page 1 to clipboard (default)
pdfyank document.pdf

# Save to file
pdfyank document.pdf -o output.png

# Specify page and DPI
pdfyank document.pdf -p 3 -d 600 -o page3.png
```

## Options

| Flag | Description | Default |
|------|-------------|---------|
| `-p, --page` | 1-based page number | `1` |
| `-d, --dpi` | Rendering DPI | `300` |
| `-o, --output` | Output PNG path | clipboard only |
| `-c, --clipboard` | Copy to clipboard | on when `-o` is omitted |
| `-q, --quiet` | Suppress status messages | off |

## License

MIT
