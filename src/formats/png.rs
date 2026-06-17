use flate2::read::ZlibDecoder;
use minifb::{self, Window, WindowOptions};
use std::fs;
use std::io::Read;

pub struct Png {
    image: Vec<u8>,
    width: u32,
    height: u32,
    bit_depth: u8,
    color_type: u8,
    compression: u8,
    filter: u8,
    interlace: u8,
    idat_data: Vec<u8>,
    decompressed_data: Vec<u8>,
    pixels: Vec<u32>,
    reconstructed_data: Vec<u8>,
    palette: Vec<u8>,
    tRNS: Vec<u8>,
}

impl Png {
    pub fn new(path: &str) -> Self {
        Self {
            image: fs::read(path).unwrap(),
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
        }
    }

    fn check_signature(&mut self) {
        if self.image[0..8] != [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
            panic!("Not a valid png");
        }
    }

    fn parse_ihdr(&mut self) {
        let mut pos = 16;

        self.width = u32::from_be_bytes(self.image[pos..pos + 4].try_into().unwrap());
        pos += 4;

        self.height = u32::from_be_bytes(self.image[pos..pos + 4].try_into().unwrap());
        pos += 4;

        self.bit_depth = self.image[pos];
        pos += 1;

        self.color_type = self.image[pos];
        pos += 1;

        self.compression = self.image[pos];
        pos += 1;

        if self.compression != 0 {
            println!("{}", self.compression);
            panic!("not supported compression");
        }

        self.filter = self.image[pos];
        pos += 1;

        self.interlace = self.image[pos];

        if self.interlace != 0 {
            panic!("interlace idk");
        }
    }

    fn bytes_per_pixel(&self) -> usize {
        match self.color_type {
            0 => 1, // grayscale
            2 => 3, // rgb
            3 => 1, // indexed
            4 => 2, // grayscale + alpha
            6 => 4, // rgba
            _ => panic!("invalid bit depth {}", self.bit_depth),
        }
    }

    fn parse_chunks(&mut self) {
        let mut pos = 8;
        while pos < self.image.len() {
            let len = u32::from_be_bytes(self.image[pos..pos + 4].try_into().unwrap());
            pos += 4;

            let chunk_type: [u8; 4] = self.image[pos..pos + 4].try_into().unwrap();
            pos += 4;

            let data: Vec<u8> = self.image[pos..pos + len as usize].try_into().unwrap();
            pos += len as usize;

            match &chunk_type {
                b"IDAT" => self.idat_data.extend_from_slice(&data),
                b"PLTE" => self.palette = data,
                b"tRNS" => self.tRNS.extend_from_slice(&data),
                b"IEND" => break,
                _ => (),
            }
            pos += 4;
        }
    }

    fn decompress_data(&mut self) {
        let mut decoder = ZlibDecoder::new(&self.idat_data[..]);

        decoder.read_to_end(&mut self.decompressed_data);
    }

    fn paeth_predictor(&mut self, a: u8, b: u8, c: u8) -> u8 {
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

    fn reconstruct_scanlines(&mut self) {
        let bpp = self.bytes_per_pixel();

        let mut previus_row = vec![0u8; self.bytes_per_pixel() * self.width as usize];

        let scanline_len = 1 + self.width * self.bytes_per_pixel() as u32;

        for row in 0..self.height {
            let mut current_row = vec![0u8; self.bytes_per_pixel() * self.width as usize];

            let pos = row * scanline_len;
            let filter = self.decompressed_data[pos as usize];

            for byte in 1..scanline_len {
                let idx = byte as usize - 1;

                let pos = byte + row * scanline_len;

                let raw_byte = self.decompressed_data[pos as usize];

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
                    4 => raw_byte.wrapping_add(self.paeth_predictor(left, above, upper_left)),
                    _ => panic!("invalid filter"),
                }
            }
            self.reconstructed_data.extend_from_slice(&current_row);
            previus_row = current_row;
        }
    }

    fn decode_pixels(&mut self) {
        match self.color_type {
            0 => self.decode_grayscale(),
            2 => self.decode_rgb(),
            3 => self.decode_index(),
            4 => self.decode_grayscale_alpha(),
            6 => self.decode_rgba(),
            _ => panic!("not supporting index color type yet"),
        }
    }

    fn decode_grayscale(&mut self) {
        self.pixels.reserve(self.reconstructed_data.len());
        for gray in &self.reconstructed_data {
            let full: u32 =
                (0xFF << 24) | ((*gray as u32) << 16) | ((*gray as u32) << 8) | *gray as u32;

            self.pixels.push(full);
        }
    }

    fn decode_rgb(&mut self) {
        self.pixels.reserve(self.reconstructed_data.len() / 3);
        for pixel in self.reconstructed_data.chunks_exact(3) {
            let r = pixel[0] as u32;
            let g = pixel[1] as u32;
            let b = pixel[2] as u32;

            let full: u32 = (0xFF << 24) | (r << 16) | (g << 8) | b;

            self.pixels.push(full);
        }
    }

    fn decode_grayscale_alpha(&mut self) {
        self.pixels.reserve(self.reconstructed_data.len() / 2);
        for pixel in self.reconstructed_data.chunks_exact(2) {
            let gray = pixel[0];
            let alpha = pixel[1];

            let full: u32 =
                ((alpha as u32) << 24) | ((gray as u32) << 16) | ((gray as u32) << 8) | gray as u32;

            self.pixels.push(full);
        }
    }

    fn decode_rgba(&mut self) {
        self.pixels.reserve(self.reconstructed_data.len() / 4);
        for pixel in self.reconstructed_data.chunks_exact(4) {
            let r = pixel[0] as u32;
            let g = pixel[1] as u32;
            let b = pixel[2] as u32;
            let a = pixel[3] as u32;

            let full: u32 = (a << 24) | (r << 16) | (g << 8) | b;

            self.pixels.push(full);
        }
    }

    fn decode_index(&mut self) {
        self.pixels.reserve(self.reconstructed_data.len());
        for &pixel in &self.reconstructed_data {
            let palette_pos = pixel as usize * 3;

            let r = self.palette[palette_pos] as u32;
            let g = self.palette[palette_pos + 1] as u32;
            let b = self.palette[palette_pos + 2] as u32;

            let a: u32 = if self.tRNS.len() > pixel as usize {
                self.tRNS[pixel as usize] as u32
            } else {
                255
            };

            let full: u32 = (a << 24) | (r << 16) | (g << 8) | b;

            self.pixels.push(full);
        }
    }

    pub fn parse(&mut self) {
        self.check_signature();
        self.parse_ihdr();
        self.parse_chunks();
        self.decompress_data();
        self.reconstruct_scanlines();
        self.decode_pixels();
    }

    pub fn draw(&mut self) {
        let image_w = self.width as usize;
        let image_h = self.height as usize;

        let mut window = Window::new(
            "image",
            image_w.min(1600),
            image_h.min(900),
            WindowOptions {
                resize: true,
                scale: minifb::Scale::X1,
                ..WindowOptions::default()
            },
        )
        .unwrap();

        window.set_target_fps(60);

        window
            .update_with_buffer(&self.pixels, self.width as usize, self.height as usize)
            .unwrap();

        while window.is_open() {
            let (window_w, window_h) = window.get_size();

            let scale_x = window_w as f32 / image_w as f32;
            let scale_y = window_h as f32 / image_h as f32;
            let scale = scale_x.min(scale_y).min(1.0);

            let draw_w = ((image_w as f32 * scale) as usize).max(1);
            let draw_h = ((image_h as f32 * scale) as usize).max(1);

            let mut buffer = vec![0u32; window_h * window_w];

            for y in 0..draw_h {
                let src_y = (y * image_h / draw_h).min(image_h - 1);

                for x in 0..draw_w {
                    let src_x = (x * image_w / draw_w).min(image_w - 1);

                    let pixel = self.pixels[src_y * image_w + src_x];

                    let offset_x = window_w.saturating_sub(draw_w) / 2;
                    let offset_y = window_h.saturating_sub(draw_h) / 2;

                    let dist_x = offset_x + x;
                    let dist_y = offset_y + y;

                    buffer[dist_y * window_w + dist_x] = pixel;
                }
            }

            window
                .update_with_buffer(&buffer, window_w, window_h)
                .unwrap();
        }
    }
}
