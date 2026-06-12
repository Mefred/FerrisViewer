mod formats;
use formats::bmp::Bmp;
use formats::png::Png;
use formats::tga::Tga;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("No file provided. Drag a file onto the executable.");
        return;
    }

    let path = &args[1];

    let file_type: String = path
        .chars()
        .rev()
        .take(3)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    if file_type.to_lowercase() == "png" {
        let mut image = Png::new(&path);
        image.parse();
        image.draw();
    } else if file_type.to_lowercase() == "bmp" {
        let mut image = Bmp::new(&path);
        image.parse();
        image.draw();
    } else if file_type.to_lowercase() == "tga" {
        let mut image = Tga::new(&path);
        image.parse();
        image.draw();
    }
}
