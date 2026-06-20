# FerrisViewer

FerrisViewer is a lightweight image viewer written entirely in rust.
It uses custom-build image format parsers instead of external image decoding libraries, implementing the decoding pipeline manually for supported formats.

The goal of this project is to explore binary file formats, compression, image reconstruction, and low level graphics handling in rust.

## Features

### Supported file formats

#### PNG
Custom PNG decoder with support for:

- PNG signature validation
- IHDR parsing
- Chunk parsing
- IDAT extracting
- Zlib decompression
- Scanline reconstruction
- PNG filtering algorithms:
  - None
  - Sub
  - Up
  - Average
  - Paeth

Supported Color formats:

- 8-bit RGB
- 8-bit RGBA
- 1/2/4/8-bit Grayscale
- 8-bit Grayscale + Alpha
- 1/2/4/8-bit Indexed Color (palette-based PNGs)

Additional features:
- Transparency through `tRNS`
- Dynamic window rescaling
- Image scaling to fit the window

#### BMP
Supports:

- 24-bit uncompressed RGB BMP
- 32-bit uncompressed RGBA BMP
- Windows Bitmap row padding handling

#### TGA
Supports:

- 24-bit True-Color TGA
- 32-bit True-Color TGA

## Usage

FerrisViewer can be used through the command line or by dragging an image file directly onto the executable.

Supported file extensions:

```
.png
.bmp
.tga
```

## Command Line Usage

Open an image by passing its path:

```bash
FerrisViewer path/to/image.png
```

Example:

```bash
cargo run -- example.png
```

## Drag and Drop

On Windows, image files can be dragged directly onto the executable.

If an image fails to load, run FerrisViewer through the command line to view the error output.

## Limitations

FerrisViewer uses custom image decoders written for learning and experimentation, so some advanced features are currently unsupported.

### PNG
Currently unsupported:

- Interlaced PNG images (Adam7)
- 16-bit color depth
- Additional compression methods outside the PNG standard defaults

### TGA

Currently unsupported:

- RLE compressed TGA images

### File Detection

The application currently identifies files using their final three-character extension:

```
png
bmp
tga
```

Files with incorrect extensions may not be detected correctly.

## Building

### Requirements

- Rust
- Cargo

## Clone Repository

```bash
git clone https://github.com/Mefred/FerrisViewer.git

cd FerrisViewer
```

## Compile

Debug build:

```bash
cargo build
```

Release build:

```bash
cargo build --release
```

## Run

### Linux

```bash
./FerrisViewer path/to/image.png
```

### Windows

```powershell
.\FerrisViewer.exe path/to/image.png
```
