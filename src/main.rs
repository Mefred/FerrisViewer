mod app;
mod formats;
use formats::bmp::Bmp;
use formats::tga::Tga;
use std::env;

fn main() -> iced::Result {
    let path = env::args().nth(1).expect("Usage: ferrisviewer <image>");

    iced::application(
        move || app::FerrisViewer::new(path.clone()),
        app::FerrisViewer::update,
        app::FerrisViewer::view,
    )
    .run()
}
