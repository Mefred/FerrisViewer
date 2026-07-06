use std::fs;
use std::path::PathBuf;

pub struct Tga {
    image: Vec<u8>,
    pub width: u16,
    pub height: u16,
    bits_per_pixel: u8,
    flip: bool,
    pub pixels: Vec<u8>,
}

impl Tga {
    pub fn new(path: PathBuf) -> Self {
        Self {
            image: fs::read(path).unwrap(),
            width: 0,
            height: 0,
            bits_per_pixel: 0,
            flip: false,
            pixels: Vec::new(),
        }
    }

    pub fn parse(&mut self) {
        if self.image[2] != 2 {
            panic!("unsuported image type");
        }

        self.bits_per_pixel = self.image[16];

        if self.bits_per_pixel != 24 && self.bits_per_pixel != 32 {
            println!("{}", self.bits_per_pixel);
            panic!("not supported pixel bit size");
        }

        if self.image[1] != 0 {
            panic!("unsuported color maps");
        }

        self.width = u16::from_le_bytes([self.image[12], self.image[13]]);
        self.height = u16::from_le_bytes([self.image[14], self.image[15]]);

        self.flip = self.image[17] == 1;

        let bits_per_pixel = if self.bits_per_pixel == 32 {
            4
        } else if self.bits_per_pixel == 24 {
            3
        } else {
            panic!("idk how it got this far")
        };

        let start = 18 + self.image[0] as usize;

        for row in 0..self.height {
            for pixel in 0..self.width {
                let pos: usize = start
                    + (row as usize) * ((self.width as usize) * bits_per_pixel)
                    + (pixel as usize) * bits_per_pixel;

                let b = self.image[pos];
                let g = self.image[pos + 1];
                let r = self.image[pos + 2];

                if self.bits_per_pixel == 32 {
                    let a = self.image[pos + 3];

                    self.pixels.push(r);
                    self.pixels.push(g);
                    self.pixels.push(b);
                    self.pixels.push(a);
                } else {
                    self.pixels.push(r);
                    self.pixels.push(g);
                    self.pixels.push(b);
                    self.pixels.push(0xFF);
                }
            }
        }
    }
}
