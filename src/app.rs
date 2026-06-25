use crate::formats::bmp::Bmp;
use crate::formats::png::Png;
use crate::formats::tga::Tga;
use iced::Task;
use iced::widget::{button, column, container, image, slider, text};
use iced::{ContentFit, Element, Fill, Rotation};
use rfd;
use std::path::PathBuf;

pub struct FerrisViewer {
    image: Option<image::Handle>,

    width: f32,
    rotation: Rotation,
    content_fit: ContentFit,
}

#[derive(Debug, Clone)]
pub enum Message {
    WidthChanged(f32),
    OpenPressed,
    ImageSelected(Option<PathBuf>),
}

impl FerrisViewer {
    pub fn new(path: PathBuf) -> Self {
        let mut viewer = Self {
            image: None,
            width: 400.0,
            rotation: Rotation::default(),
            content_fit: ContentFit::Contain,
        };

        viewer.open_image(path);

        viewer
    }
}

impl FerrisViewer {
    fn open_image(&mut self, path: PathBuf) {
        match path.extension().and_then(|s| s.to_str()) {
            Some("png") => match Png::new(path) {
                Ok(mut img) => match img.parse() {
                    Ok(_) => {
                        self.image =
                            Some(image::Handle::from_rgba(img.width, img.height, img.pixels))
                    }
                    Err(e) => println!("Failed to load image: {:?}", e),
                },
                Err(e) => println!("Failed to load image: {:?}", e),
            },
            Some("bmp") => {
                let mut img = Bmp::new(path);
                img.parse();
                self.image = Some(image::Handle::from_rgba(img.width, img.height, img.pixels));
            }
            Some("tga") => {
                let mut img = Tga::new(path);
                img.parse();
                self.image = Some(image::Handle::from_rgba(
                    img.width as u32,
                    img.height as u32,
                    img.pixels,
                ));
            }
            None => (),
            _ => panic!("Not a supported file extension"),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WidthChanged(width) => {
                self.width = width;
                Task::none()
            }

            Message::OpenPressed => Task::perform(
                async {
                    rfd::FileDialog::new()
                        .add_filter("Images", &["png", "bmp", "tga"])
                        .pick_file()
                },
                Message::ImageSelected,
            ),

            Message::ImageSelected(Some(path)) => {
                self.open_image(path);
                Task::none()
            }

            Message::ImageSelected(None) => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let controls = column![
            button("Open").on_press(Message::OpenPressed),
            slider(100.0..=1000.0, self.width, Message::WidthChanged),
            text(format!("Width: {}", self.width))
        ]
        .spacing(10);

        let image_area: Element<Message> = if let Some(handle) = &self.image {
            container(
                image(handle.clone())
                    .width(self.width)
                    .content_fit(self.content_fit)
                    .rotation(self.rotation),
            )
            .center_x(Fill)
            .center_y(Fill)
            .width(Fill)
            .height(Fill)
            .into()
        } else {
            container(text("Open a image"))
                .center_x(Fill)
                .center_y(Fill)
                .width(Fill)
                .height(Fill)
                .into()
        };

        container(column![controls, image_area])
            .padding(20)
            .width(Fill)
            .height(Fill)
            .into()
    }
}
