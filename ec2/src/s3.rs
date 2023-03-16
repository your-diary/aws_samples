use std::{error::Error, time::SystemTime};

use bytes::Bytes;
use s3::creds::Credentials;

pub struct S3 {
    bucket_name: String,
    client: aws_sdk_s3::Client,

    //The official `aws_sdk_s3::Client` doesn't support creating a presigned URL for an object.
    //So we additionally use unofficial `rust-s3` crate, which exposes `s3` module.
    bucket: s3::Bucket,
}

impl S3 {
    pub async fn new(bucket_name: String) -> Result<Self, Box<dyn Error>> {
        let config = aws_config::load_from_env().await;
        let client = aws_sdk_s3::Client::new(&config);

        let bucket = s3::Bucket::new(
            &bucket_name,
            s3::Region::ApNortheast1,
            Credentials::default()?,
        )?;

        Ok(Self {
            bucket_name,
            client,
            bucket,
        })
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

    pub fn get_presigned_url(
        &self,
        filename: &str,
        expiration_secs: u32,
    ) -> Result<String, Box<dyn Error>> {
        let res = self.bucket.presign_get(filename, expiration_secs, None);
        if let Err(e) = res {
            println!("{:?}", e);
            Err("failed to create a presigned URL".into())
        } else {
            Ok(res.unwrap())
        }
    }
}
