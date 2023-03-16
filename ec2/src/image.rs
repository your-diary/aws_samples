use bytes::Bytes;
use image::{codecs::png::PngEncoder, ColorType, ImageEncoder, Rgb, RgbImage};

use super::color::Color;

pub struct Image;

impl Image {
    pub fn create_image(width: u32, height: u32, color: &Color) -> Bytes {
        let mut img = RgbImage::new(width, height);
        for x in 0..width {
            for y in 0..height {
                let r = color.r;
                let g = color.g;
                let b = color.b;
                img.put_pixel(x, y, Rgb([r, g, b]));
            }
        }

        let mut buf = Vec::new();
        let encoder = PngEncoder::new(&mut buf);
        encoder
            .write_image(&img, width, height, ColorType::Rgb8)
            .unwrap();
        buf.into()
    }
}
