use flate2::read::ZlibDecoder;
use minifb::{self, Window, WindowOptions};
use std::cmp::min;
use std::fs;
use std::io::Read;
use std::path::absolute;

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

        if self.bit_depth != 8 {
            println!("{}", self.bit_depth);
            panic!("not supported bit depth");
        }

        self.color_type = self.image[pos];
        pos += 1;

        if self.color_type != 2 {
            println!("{}", self.color_type);
            panic!("not supported color type");
        }

        self.compression = self.image[pos];
        pos += 1;

        if self.compression != 0 {
            println!("{}", self.compression);
            panic!("not supported compression");
        }

        self.filter = self.image[pos];
        pos += 1;

        self.interlace = self.image[pos];
        pos += 1;

        if self.interlace != 0 {
            panic!("interlace idk");
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

    fn parse_decompressed_data(&mut self) {
        self.pixels.reserve((self.width * self.height) as usize);
        let mut previus_row = vec![0u8; 3 * self.width as usize];
        for row in 0..self.height {
            let pos = row * (1 + self.width * 3);
            let filter = self.decompressed_data[pos as usize];

            let mut left_r = 0u8;
            let mut left_g = 0u8;
            let mut left_b = 0u8;

            let mut current_row = vec![0u8; 3 * self.width as usize];
            for unit in 0..self.width {
                let pos = row * (1 + self.width * 3) + 1 + unit * 3;

                let r = self.decompressed_data[pos as usize];
                let g = self.decompressed_data[1 + pos as usize];
                let b = self.decompressed_data[2 + pos as usize];

                if filter == 0 {
                    let full: u32 =
                        ((0xFF << 24) | (r as u32) << 16) | ((g as u32) << 8) | b as u32;
                    self.pixels.push(full);

                    current_row[3 * unit as usize] = r;
                    current_row[1 + 3 * unit as usize] = g;
                    current_row[2 + 3 * unit as usize] = b;
                } else if filter == 1 {
                    left_r = left_r.wrapping_add(r);
                    left_g = left_g.wrapping_add(g);
                    left_b = left_b.wrapping_add(b);

                    let full: u32 = ((0xFF << 24) | (left_r as u32) << 16)
                        | ((left_g as u32) << 8)
                        | left_b as u32;

                    self.pixels.push(full);

                    current_row[3 * unit as usize] = left_r;
                    current_row[1 + 3 * unit as usize] = left_g;
                    current_row[2 + 3 * unit as usize] = left_b;
                } else if filter == 2 {
                    let r2 = previus_row[3 * unit as usize].wrapping_add(r);
                    let g2 = previus_row[1 + 3 * unit as usize].wrapping_add(g);
                    let b2 = previus_row[2 + 3 * unit as usize].wrapping_add(b);

                    let full: u32 =
                        ((0xFF << 24) | (r2 as u32) << 16) | ((g2 as u32) << 8) | b2 as u32;

                    self.pixels.push(full);

                    current_row[3 * unit as usize] = r2;
                    current_row[1 + 3 * unit as usize] = g2;
                    current_row[2 + 3 * unit as usize] = b2;
                } else if filter == 3 {
                    let above_r = previus_row[3 * unit as usize];
                    let above_g = previus_row[1 + 3 * unit as usize];
                    let above_b = previus_row[2 + 3 * unit as usize];

                    let pr = ((left_r as u16 + above_r as u16) / 2) as u8;
                    let pg = ((left_g as u16 + above_g as u16) / 2) as u8;
                    let pb = ((left_b as u16 + above_b as u16) / 2) as u8;

                    let r2 = r.wrapping_add(pr);
                    let g2 = g.wrapping_add(pg);
                    let b2 = b.wrapping_add(pb);

                    let full: u32 =
                        ((0xFF << 24) | (r2 as u32) << 16) | ((g2 as u32) << 8) | b2 as u32;

                    self.pixels.push(full);

                    left_r = r2;
                    left_g = g2;
                    left_b = b2;

                    current_row[3 * unit as usize] = r2;
                    current_row[1 + 3 * unit as usize] = g2;
                    current_row[2 + 3 * unit as usize] = b2;
                } else if filter == 4 {
                    let closest = self.paeth_predictor(
                        left_r,
                        previus_row[3 * unit as usize],
                        if row == 0 {
                            0
                        } else if unit == 0 {
                            0
                        } else {
                            previus_row[3 * (unit - 1) as usize]
                        },
                    );

                    let r2 = r.wrapping_add(closest);

                    let closest = self.paeth_predictor(
                        left_g,
                        previus_row[1 + 3 * unit as usize],
                        if row == 0 {
                            0
                        } else if unit == 0 {
                            0
                        } else {
                            previus_row[1 + 3 * (unit - 1) as usize]
                        },
                    );

                    let g2 = g.wrapping_add(closest);

                    let closest = self.paeth_predictor(
                        left_b,
                        previus_row[2 + 3 * unit as usize],
                        if row == 0 {
                            0
                        } else if unit == 0 {
                            0
                        } else {
                            previus_row[2 + 3 * (unit - 1) as usize]
                        },
                    );

                    let b2 = b.wrapping_add(closest);

                    let full: u32 =
                        ((0xFF << 24) | (r2 as u32) << 16) | ((g2 as u32) << 8) | b2 as u32;

                    self.pixels.push(full);

                    left_r = r2;
                    left_g = g2;
                    left_b = b2;

                    current_row[3 * unit as usize] = r2;
                    current_row[1 + 3 * unit as usize] = g2;
                    current_row[2 + 3 * unit as usize] = b2;
                } else {
                    self.pixels.push(0);
                }
            }
            previus_row = current_row;
        }
    }

    pub fn parse(&mut self) {
        self.check_signature();
        self.parse_ihdr();
        self.parse_chunks();
        self.decompress_data();
        self.parse_decompressed_data();
    }

    pub fn draw(&mut self) {
        let mut window = Window::new(
            "image",
            self.width as usize,
            self.height as usize,
            WindowOptions {
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
            window.update();
        }
    }
}
