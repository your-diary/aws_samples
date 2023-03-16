use std::{error::Error, time::SystemTime};

use bytes::Bytes;

pub struct S3 {
    bucket_name: String,
    client: aws_sdk_s3::Client,
}

impl S3 {
    pub async fn new(bucket_name: String) -> Self {
        let config = aws_config::load_from_env().await;
        let client = aws_sdk_s3::Client::new(&config);
        Self {
            bucket_name,
            client,
        }
    }

    pub fn create_filename() -> String {
        format!(
            "{}.png",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        )
    }

    pub async fn upload(&self, filename: &str, image: Bytes) -> Result<(), Box<dyn Error>> {
        let res = self
            .client
            .put_object()
            .bucket(&self.bucket_name)
            .body(image.into())
            .key(filename)
            .content_type("image/png")
            .send()
            .await;
        if let Err(e) = res {
            println!("{}", e);
            Err("upload failed".into())
        } else {
            Ok(())
        }
    }
}
