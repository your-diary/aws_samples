use std::error::Error;

use ec2::image::Image;
use ec2::s3::S3;

const BUCKET_NAME: &str = "bucket-test-001-a";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let width = 200;
    let height = 100;
    let image = Image::create_image(width, height);

    let s3 = S3::new(BUCKET_NAME.to_string()).await;
    let res = s3.upload(&S3::create_filename(), image).await;
    println!("{:?}", res);
    res
}
