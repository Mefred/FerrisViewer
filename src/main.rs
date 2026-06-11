mod formats;
use formats::bmp::Image;

fn main() {
    let mut imag = Image::new("mega.bmp");
    imag.parse();
    imag.draw();
}
