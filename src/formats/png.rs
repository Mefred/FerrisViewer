use flate2::read::ZlibDecoder;
use minifb::{self, Window, WindowOptions};
use std::fs;
use std::io::Read;

#[derive(Debug)]
pub enum PngError {
    InvalidSignature,
    UnsupportedBitDepth(u8),
    UnsupportedBitDepthForGrayscale(u8),
    UnsupportedColorType(u8),
    UnsupportedCompression(u8),
    UnsupportedInterlace(u8),
    UnsupportedFilter(u8),
    UnexpectedEndOfFile,
    CorruptChunk,
    DecompressionFailed,
    FileReadFailed,
}

pub struct Png {
    image: Vec<u8>,
    pub width: u32,
    pub height: u32,
    bit_depth: u8,
    color_type: u8,
    compression: u8,
    filter: u8,
    interlace: u8,
    idat_data: Vec<u8>,
    decompressed_data: Vec<u8>,
    pub pixels: Vec<u8>,
    reconstructed_data: Vec<u8>,
    palette: Vec<u8>,
    tRNS: Vec<u8>,
}

impl Png {
    pub fn new(path: String) -> Result<Self, PngError> {
        Ok(Self {
            image: fs::read(path).map_err(|_| PngError::FileReadFailed)?,
            width: 0,
            height: 0,
            bit_depth: 0,
            color_type: 0,
            compression: 0,
            filter: 0,
            interlace: 0,
            idat_data: Vec::new(),
            decompressed_data: Vec::new(),
            pixels: Vec::new(),
            reconstructed_data: Vec::new(),
            palette: Vec::new(),
            tRNS: Vec::new(),
        })
    }

    fn check_signature(&mut self) -> Result<(), PngError> {
        if self.image.len() < 8 {
            return Err(PngError::UnexpectedEndOfFile);
        }

        if self.image[0..8] != [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
            return Err(PngError::InvalidSignature);
        }
        Ok(())
    }

    fn parse_ihdr(&mut self) -> Result<(), PngError> {
        if self.image.len() < 33 {
            return Err(PngError::UnexpectedEndOfFile);
        }

        let mut pos = 16;

        self.width = u32::from_be_bytes(
            self.image[pos..pos + 4]
                .try_into()
                .map_err(|_| PngError::UnexpectedEndOfFile)?,
        );
        pos += 4;

        self.height = u32::from_be_bytes(
            self.image[pos..pos + 4]
                .try_into()
                .map_err(|_| PngError::UnexpectedEndOfFile)?,
        );
        pos += 4;

        self.bit_depth = self.image[pos];
        pos += 1;

        match self.bit_depth {
            1 | 2 | 4 | 8 => (),
            16 => return Err(PngError::UnsupportedBitDepth(16)),
            _ => return Err(PngError::UnsupportedBitDepth(self.bit_depth)),
        }

        self.color_type = self.image[pos];
        pos += 1;

        match self.color_type {
            0 | 2 | 3 | 4 | 6 => (),
            _ => return Err(PngError::UnsupportedColorType(self.color_type)),
        }

        self.compression = self.image[pos];
        pos += 1;

        if self.compression != 0 {
            return Err(PngError::UnsupportedCompression(self.compression));
        }

        self.filter = self.image[pos];
        pos += 1;

        self.interlace = self.image[pos];

        if self.interlace != 0 {
            return Err(PngError::UnsupportedInterlace(self.interlace));
        }

        Ok(())
    }

    fn bytes_per_pixel(&self) -> Result<usize, PngError> {
        match self.color_type {
            0 => Ok(1), // grayscale
            2 => Ok(3), // rgb
            3 => Ok(1), // indexed
            4 => Ok(2), // grayscale + alpha
            6 => Ok(4), // rgba
            _ => Err(PngError::UnsupportedColorType(self.color_type)),
        }
    }

    fn parse_chunks(&mut self) -> Result<(), PngError> {
        let mut pos = 8;
        while pos < self.image.len() {
            if pos + 8 > self.image.len() {
                return Err(PngError::UnexpectedEndOfFile);
            }

            let len = u32::from_be_bytes(
                self.image[pos..pos + 4]
                    .try_into()
                    .map_err(|_| PngError::CorruptChunk)?,
            );
            pos += 4;

            let chunk_type: [u8; 4] = self.image[pos..pos + 4]
                .try_into()
                .map_err(|_| PngError::CorruptChunk)?;
            pos += 4;

            if pos + len as usize > self.image.len() {
                return Err(PngError::UnexpectedEndOfFile);
            }
            let data: Vec<u8> = self.image[pos..pos + len as usize].to_vec();
            pos += 4 + len as usize;

            match &chunk_type {
                b"IDAT" => self.idat_data.extend_from_slice(&data),
                b"PLTE" => self.palette = data,
                b"tRNS" => self.tRNS.extend_from_slice(&data),
                b"IEND" => break,
                _ => (),
            }
        }
        Ok(())
    }

    fn decompress_data(&mut self) -> Result<(), PngError> {
        let mut decoder = ZlibDecoder::new(&self.idat_data[..]);

        decoder
            .read_to_end(&mut self.decompressed_data)
            .map_err(|_| PngError::DecompressionFailed)?;

        Ok(())
    }

    fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
        let left = a as i32;
        let above = b as i32;
        let upper_left = c as i32;

        let pr = left + above - upper_left;
        let p_left_r = pr.abs_diff(left);
        let p_above_r = pr.abs_diff(above);
        let p_upper_left = pr.abs_diff(upper_left);

        let closest = if p_left_r <= p_upper_left && p_left_r <= p_above_r {
            left
        } else if p_above_r <= p_upper_left {
            above
        } else {
            upper_left
        };

        return closest as u8;
    }

    fn scanline_data_bytes(&self) -> Result<usize, PngError> {
        match self.color_type {
            0 => Ok((self.width as usize * self.bit_depth as usize + 7) / 8),
            2 => Ok(self.width as usize * 3 * (self.bit_depth as usize / 8)),
            3 => Ok((self.width as usize * self.bit_depth as usize + 7) / 8),
            4 => Ok(self.width as usize * 2 * (self.bit_depth as usize / 8)),
            6 => Ok(self.width as usize * 4 * (self.bit_depth as usize / 8)),
            _ => Err(PngError::UnsupportedColorType(self.color_type)),
        }
    }

    fn reconstruct_scanlines(&mut self) -> Result<(), PngError> {
        let bpp = self.bytes_per_pixel()?;

        let mut previus_row = vec![0u8; self.scanline_data_bytes()?];

        let scanline_len = 1 + self.scanline_data_bytes()? as u32;

        for row in 0..self.height {
            let mut current_row = vec![0u8; self.scanline_data_bytes()?];

            let pos = row * scanline_len;
            let filter = *self
                .decompressed_data
                .get(pos as usize)
                .ok_or(PngError::UnexpectedEndOfFile)?;

            for byte in 1..scanline_len {
                let idx = byte as usize - 1;

                let pos = byte + row * scanline_len;

                let raw_byte = *self
                    .decompressed_data
                    .get(pos as usize)
                    .ok_or(PngError::UnexpectedEndOfFile)?;

                let left = if idx >= bpp {
                    current_row[idx - bpp]
                } else {
                    0
                };

                let above = previus_row[idx];

                let upper_left = if idx >= bpp {
                    previus_row[idx - bpp]
                } else {
                    0
                };

                current_row[idx] = match filter {
                    0 => raw_byte,
                    1 => raw_byte.wrapping_add(left),
                    2 => raw_byte.wrapping_add(above),
                    3 => raw_byte.wrapping_add(((left as u16 + above as u16) / 2) as u8),
                    4 => raw_byte.wrapping_add(Png::paeth_predictor(left, above, upper_left)),
                    _ => return Err(PngError::UnsupportedFilter(filter)),
                }
            }
            self.reconstructed_data.extend_from_slice(&current_row);
            previus_row = current_row;
        }
        Ok(())
    }

    fn decode_pixels(&mut self) -> Result<(), PngError> {
        match self.color_type {
            0 => return self.decode_grayscale(),
            2 => return self.decode_rgb(),
            3 => return self.decode_index(),
            4 => return self.decode_grayscale_alpha(),
            6 => return self.decode_rgba(),
            _ => return Err(PngError::UnsupportedColorType(self.color_type)),
        }
    }

    fn decode_grayscale(&mut self) -> Result<(), PngError> {
        self.pixels.reserve(self.reconstructed_data.len());
        for &gray in &self.reconstructed_data {
            let gray = match self.bit_depth {
                1 => gray * 255,
                2 => gray * 85,
                4 => gray * 17,
                8 => gray,
                _ => return Err(PngError::UnsupportedBitDepthForGrayscale(self.bit_depth)),
            };

            self.pixels.push(gray);
            self.pixels.push(gray);
            self.pixels.push(gray);
            self.pixels.push(0xFF);
        }
        Ok(())
    }

    fn decode_rgb(&mut self) -> Result<(), PngError> {
        self.pixels.reserve(self.reconstructed_data.len() / 3);
        for pixel in self.reconstructed_data.chunks_exact(3) {
            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];

            self.pixels.push(r);
            self.pixels.push(g);
            self.pixels.push(b);
            self.pixels.push(0xFF);
        }
        Ok(())
    }

    fn decode_grayscale_alpha(&mut self) -> Result<(), PngError> {
        self.pixels.reserve(self.reconstructed_data.len() / 2);
        for pixel in self.reconstructed_data.chunks_exact(2) {
            let gray = pixel[0];
            let alpha = pixel[1];

            self.pixels.push(gray);
            self.pixels.push(gray);
            self.pixels.push(gray);
            self.pixels.push(alpha);
        }
        Ok(())
    }

    fn decode_rgba(&mut self) -> Result<(), PngError> {
        self.pixels.reserve(self.reconstructed_data.len() / 4);
        for pixel in self.reconstructed_data.chunks_exact(4) {
            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];
            let a = pixel[3];

            self.pixels.push(r);
            self.pixels.push(g);
            self.pixels.push(b);
            self.pixels.push(a);
        }
        Ok(())
    }

    fn decode_index(&mut self) -> Result<(), PngError> {
        self.pixels.reserve(self.reconstructed_data.len());
        for &pixel in &self.reconstructed_data {
            let palette_pos = pixel as usize * 3;

            if self.palette.len() < palette_pos + 3 {
                return Err(PngError::UnexpectedEndOfFile);
            }

            let r = self.palette[palette_pos];
            let g = self.palette[palette_pos + 1];
            let b = self.palette[palette_pos + 2];

            let a: u8 = if self.tRNS.len() > pixel as usize {
                self.tRNS[pixel as usize]
            } else {
                255
            };

            self.pixels.push(r);
            self.pixels.push(g);
            self.pixels.push(b);
            self.pixels.push(a);
        }
        Ok(())
    }

    fn unpack_pixels(&mut self) -> Result<(), PngError> {
        let packed = self.reconstructed_data.clone();

        self.reconstructed_data = Vec::new();

        match self.bit_depth {
            1 => {
                for row in 0..self.height as usize {
                    let mut pixels_in_row = 0;
                    for byte in 0..self.scanline_data_bytes()? {
                        let pos = byte + row * self.scanline_data_bytes()?;

                        let first = packed[pos] >> 7;
                        pixels_in_row += 1;

                        self.reconstructed_data.push(first);

                        if pixels_in_row == self.width {
                            break;
                        }

                        let second = (packed[pos] & 0b01000000) >> 6;
                        pixels_in_row += 1;

                        self.reconstructed_data.push(second);

                        if pixels_in_row == self.width {
                            break;
                        }

                        let third = (packed[pos] & 0b00100000) >> 5;
                        pixels_in_row += 1;

                        self.reconstructed_data.push(third);

                        if pixels_in_row == self.width {
                            break;
                        }

                        let forth = (packed[pos] & 0b00010000) >> 4;
                        pixels_in_row += 1;

                        self.reconstructed_data.push(forth);

                        if pixels_in_row == self.width {
                            break;
                        }

                        let fith = (packed[pos] & 0b00001000) >> 3;
                        pixels_in_row += 1;

                        self.reconstructed_data.push(fith);

                        if pixels_in_row == self.width {
                            break;
                        }

                        let six = (packed[pos] & 0b00000100) >> 2;
                        pixels_in_row += 1;

                        self.reconstructed_data.push(six);

                        if pixels_in_row == self.width {
                            break;
                        }

                        let seven = (packed[pos] & 0b00000010) >> 1;
                        pixels_in_row += 1;

                        self.reconstructed_data.push(seven);

                        if pixels_in_row == self.width {
                            break;
                        }

                        let eight = packed[pos] & 0b00000001;
                        pixels_in_row += 1;

                        self.reconstructed_data.push(eight);

                        if pixels_in_row == self.width {
                            break;
                        }
                    }
                }
            }
            2 => {
                for row in 0..self.height as usize {
                    let mut pixels_in_row = 0;
                    for byte in 0..self.scanline_data_bytes()? {
                        let pos = byte + row * self.scanline_data_bytes()?;

                        let first = packed[pos] >> 6;
                        pixels_in_row += 1;

                        self.reconstructed_data.push(first);

                        if pixels_in_row == self.width {
                            break;
                        }

                        let second = (packed[pos] & 0b00110000) >> 4;
                        pixels_in_row += 1;

                        self.reconstructed_data.push(second);

                        if pixels_in_row == self.width {
                            break;
                        }

                        let third = (packed[pos] & 0b00001100) >> 2;
                        pixels_in_row += 1;

                        self.reconstructed_data.push(third);

                        if pixels_in_row == self.width {
                            break;
                        }

                        let forth = packed[pos] & 0b00000011;
                        pixels_in_row += 1;

                        self.reconstructed_data.push(forth);

                        if pixels_in_row == self.width {
                            break;
                        }
                    }
                }
            }
            4 => {
                for row in 0..self.height as usize {
                    let mut pixels_in_row = 0;
                    for byte in 0..self.scanline_data_bytes()? {
                        let pos = byte + row * self.scanline_data_bytes()?;

                        let first = packed[pos] >> 4;
                        pixels_in_row += 1;

                        self.reconstructed_data.push(first);

                        if pixels_in_row == self.width {
                            break;
                        }

                        let second = packed[pos] & 0x0F;
                        pixels_in_row += 1;

                        self.reconstructed_data.push(second);

                        if pixels_in_row == self.width {
                            break;
                        }
                    }
                }
            }
            8 => self.reconstructed_data = packed.clone(),
            _ => return Err(PngError::UnsupportedBitDepth(self.bit_depth)),
        }

        Ok(())
    }

    pub fn parse(&mut self) -> Result<(), PngError> {
        self.check_signature()?;
        self.parse_ihdr()?;
        self.parse_chunks()?;
        self.decompress_data()?;
        self.reconstruct_scanlines()?;
        self.unpack_pixels()?;
        self.decode_pixels()?;

        Ok(())
    }
}
