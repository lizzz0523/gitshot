# gitshot

Render git diff and status as PNG images.

Built for sharing code changes on mobile messaging apps where text-based git output is hard to read.

## Install

```bash
cargo install --path .
```

## Usage

### diff

Render git diff as a PNG image:

```bash
# Current directory
gitshot diff

# Specific paths
gitshot diff src/main.rs src/lib.rs

# Show whitespace changes (ignored by default)
gitshot diff -w
```

### status

Render git status as a PNG image:

```bash
# Current directory
gitshot status

# Specific paths
gitshot status src/components/
```

Output images are saved to `/tmp/gitshot_<timestamp>.png` and the path is printed to stdout.

## Configuration

gitshot reads `~/.config/gitshot/gitshot.toml` on startup. All fields are optional.

```toml
color_scheme = "dark"                          # "dark" (default) or "light"
font_path = "/System/Library/Fonts/Monaco.ttf" # Path to a .ttf or .ttc font
font_size = 13.0                               # Font size in points
line_height = 20.0                             # Line height in pixels
img_padding = 16.0                             # Canvas padding in pixels
max_img_width = 1800                           # Maximum image width in pixels
```

### Color Schemes

**Dark** (default) — GitHub Dark theme inspired colors with a dark canvas background.

**Light** — Light canvas background with adjusted text and highlight colors for readability.

## Building

```bash
cargo build --release
```

## License

MIT
