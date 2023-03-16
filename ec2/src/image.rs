use bytes::Bytes;
use image::{codecs::png::PngEncoder, ColorType, ImageEncoder, Rgb, RgbImage};

pub struct Image;

impl Image {
    pub fn create_image(width: u32, height: u32) -> Bytes {
        let mut img = RgbImage::new(width, height);
        for x in 0..width {
            for y in 0..height {
                let r = ((x + y) % 4).try_into().unwrap();
                let g = 255 - r;
                let b = 127 - r;
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
