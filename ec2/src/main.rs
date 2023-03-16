use std::error::Error;

use ec2::image::Image;
use ec2::s3::S3;

use ec2::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let width = 200;
    let height = 100;
    let image = Image::create_image(width, height);

    let filename = S3::create_filename();

    let config = Config::new("./config.json");

    let s3 = S3::new(&config.s3).await?;
    let res = s3.upload(&filename, image).await;
    println!("{:?}", res);

    let url = s3.get_presigned_url(&filename, 30);
    println!("{:?}", url);

    Ok(())
}
