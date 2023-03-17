use std::error::Error;

use bytes::Bytes;
use s3::creds::Credentials;

use super::config::S3Config;

pub struct S3 {
    bucket_name: String,
    client: aws_sdk_s3::Client,

    //The official `aws_sdk_s3::Client` doesn't support creating a presigned URL for an object.
    //So we additionally use unofficial `rust-s3` crate, which exposes `s3` module.
    bucket: s3::Bucket,
}

impl S3 {
    pub async fn new(s3_config: &S3Config) -> Result<Self, Box<dyn Error>> {
        let config = aws_config::load_from_env().await;
        let client = aws_sdk_s3::Client::new(&config);

        let bucket = s3::Bucket::new(
            &s3_config.bucket_name,
            s3::Region::ApNortheast1,
            Credentials::default()?,
        )?;

        Ok(Self {
            bucket_name: s3_config.bucket_name.clone(),
            client,
            bucket,
        })
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
