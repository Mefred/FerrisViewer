use minifb::{self, Window, WindowOptions};
use std::fs;

pub struct Bmp {
    image: Vec<u8>,
    width: u32,
    height: u32,
    pixels: Vec<u32>,
    bits_per_pixel: u16,
    pixel_offset: u32,
}

impl Bmp {
    pub fn new(path: &str) -> Self {
        Self {
            image: fs::read(path).unwrap(),
            width: 0,
            height: 0,
            pixels: Vec::new(),
            bits_per_pixel: 0,
            pixel_offset: 0,
        }
    }

    pub fn parse(&mut self) {
        if &self.image[0..2] != b"BM" {
            println!("not a bmp");
            return;
        }

        self.width = u32::from_le_bytes([
            self.image[18],
            self.image[19],
            self.image[20],
            self.image[21],
        ]);

        self.height = u32::from_le_bytes([
            self.image[22],
            self.image[23],
            self.image[24],
            self.image[25],
        ]);

        self.bits_per_pixel = u16::from_le_bytes([self.image[28], self.image[29]]);

        self.pixel_offset = u32::from_le_bytes([
            self.image[10],
            self.image[11],
            self.image[12],
            self.image[13],
        ]);

        let num_pixels = if self.bits_per_pixel == 32 {
            4
        } else if self.bits_per_pixel == 24 {
            3
        } else {
            panic!("not supported format")
        };

        let raw_row_size = self.width * num_pixels;
        let padded_row_size = ((raw_row_size + 3) / 4) * 4;

        for row in 0..self.height {
            for pixel in 0..self.width {
                let pos: usize = (self.pixel_offset
                    + (self.height - 1 - row) * padded_row_size
                    + pixel * num_pixels) as usize;
                let b = self.image[pos];
                let g = self.image[pos + 1];
                let r = self.image[pos + 2];
                if num_pixels == 4 {
                    let a = self.image[pos + 3];
                    let pixel_full: u32 =
                        ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32;
                    self.pixels.push(pixel_full);
                } else if num_pixels == 3 {
                    let pixel_full: u32 = ((r as u32) << 16) | ((g as u32) << 8) | b as u32;
                    self.pixels.push(pixel_full);
                } else {
                    panic!("idk how it got this far");
                }
            }
        }
    }

    pub fn draw(self) {
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
        window
            .update_with_buffer(&self.pixels, self.width as usize, self.height as usize)
            .unwrap();
        while window.is_open() {
            window.update();
        }
    }
}
