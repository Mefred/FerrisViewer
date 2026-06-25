mod app;
mod formats;
use std::env;
use std::path::PathBuf;

fn main() -> iced::Result {
    let _path = env::args().nth(1);
    let mut path = PathBuf::new();

    match _path {
        None => (),
        Some(other) => path = other.try_into().unwrap(),
    }

    iced::application(
        move || app::FerrisViewer::new(path.clone()),
        app::FerrisViewer::update,
        app::FerrisViewer::view,
    )
    .run()
}
