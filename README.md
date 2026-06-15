# FerrisViewer

FerrisViewer is a simple rust project for displaying images.

---

## Features
- **Supported Formats**:
  - **PNG**: Supports Zlib-compressed 8-bit truecolor (RGB) formats. Includes manual implementations for PNG scanline filters (None, Sub, Up, Average, and Paeth).
  - **BMP**: Supports 24-bit (RGB) and 32-bit (RGBA) uncompressed Windows Bitmap files, factoring in pixel data row padding.
  - **TGA**: Supports 24-bit and 32-bit uncompressed True-Color Truevision TGA images.

---

## Usage

You can use the image viewer via the Command Line Interface (CLI) or by dragging and dropping a file.

---

## Via Command Line

Pass the path of the image you want to open as an argument:

---

## Via Drag and Drop

On Windows, you can simply and drag any compatible .png, .bmp, or .tga file directly onto the executable file to view it. If you are having problems use the CLI to view the error message.

---

## Limitations

Since this viewer uses entirely custom parsers built for learning purposes:
- PNG: Interlaced PNGs, indexed colors, and greyscale bit-depths other than 8-bit are not supported.
- TGA: Compressed (RLE) TGA images are currently unsupported.
- Extensions: The application detects files strictly by their final 3-character extension (png, bmp, tga).
