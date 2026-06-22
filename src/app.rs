use crate::formats::png::{self, Png};
use iced::widget::{column, container, image, slider, text};
use iced::{Alignment, ContentFit, Element, Fill, Rotation};

pub struct FerrisViewer {
    image: image::Handle,

    width: f32,
    rotation: Rotation,
    content_fit: ContentFit,
}

#[derive(Debug, Clone)]
pub enum Message {
    WidthChanged(f32),
}

impl FerrisViewer {
    pub fn new(path: String) -> Self {
        let mut pixels: Vec<u8> = Vec::new();
        let mut width = 0;
        let mut height = 0;

        match Png::new(path) {
            Ok(mut img) => match img.parse() {
                Ok(_) => {
                    width = img.width;
                    height = img.height;
                    pixels = img.pixels;
                }
                Err(e) => println!("Failed to load image: {:?}", e),
            },
            Err(e) => println!("Failed to load image: {:?}", e),
        }

        println!(
            "expected={} actual={}",
            width as usize * height as usize * 4,
            pixels.len()
        );

        let handle = image::Handle::from_rgba(width, height, pixels);

        Self {
            image: handle,

            width: 400.0,

            rotation: Rotation::default(),

            content_fit: ContentFit::Contain,
        }
    }
}

impl FerrisViewer {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::WidthChanged(width) => {
                self.width = width;
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let viewer = image(self.image.clone())
            .width(self.width)
            .content_fit(self.content_fit)
            .rotation(self.rotation);

        let controls = column![
            slider(100.0..=1000.0, self.width, Message::WidthChanged),
            text(format!("Width: {}", self.width))
        ]
        .spacing(10);

        let image_area = container(viewer)
            .center_x(Fill)
            .center_y(Fill)
            .width(Fill)
            .height(Fill);

        container(column![controls, image_area])
            .padding(20)
            .width(Fill)
            .height(Fill)
            .into()
    }
}
