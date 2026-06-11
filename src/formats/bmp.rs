use std::fs;

// Todo:
// Fix padding
// Finish

pub struct Image {
    image: Vec<u8>,
    width: u32,
    height: u32,
    pixels: Vec<u8>,
    bits_per_pixel: u16,
    compresion: u32,
}

impl Image {
    pub fn new(path: &str) -> Self {
        Self {
            image: fs::read(path).unwrap(),
            width: 0,
            height: 0,
            pixels: Vec::new(),
            bits_per_pixel: 0,
            compresion: 0,
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

        self.compresion = u32::from_le_bytes([
            self.image[30],
            self.image[31],
            self.image[32],
            self.image[33],
        ]);

        self.bits_per_pixel = u16::from_le_bytes([self.image[28], self.image[29]]);

        if self.compresion != 0 {
            todo!("add compresion");
            return;
        }

        let mut pos = u32::from_le_bytes([
            self.image[10],
            self.image[11],
            self.image[12],
            self.image[13],
        ]);

        let mut num_times = if self.bits_per_pixel == 24 {
            self.width * self.height * 3
        } else {
            self.width * self.height * 4
        };
        num_times += pos;

        while pos <= num_times {
            let b = self.image[pos as usize];
            pos += 1;
            let g = self.image[pos as usize];
            pos += 1;
            let r = self.image[pos as usize];
            pos += 1;
            if self.bits_per_pixel == 32 {
                let a = self.image[pos as usize];
                pos += 1;
                self.pixels.push(r);
                self.pixels.push(g);
                self.pixels.push(b);
                self.pixels.push(a);
            } else {
                self.pixels.push(r);
                self.pixels.push(g);
                self.pixels.push(b);
            }
        }
    }
}
