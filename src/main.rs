mod formats;
use formats::bmp::Image;

fn main() {
    let mut imag = Image::new("frieren-4k.bmp");
    imag.parse();
}
