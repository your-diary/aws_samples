pub mod color;
pub mod config;
pub mod dynamodb;
pub mod image;
pub mod mysql;
pub mod s3;

use std::{error::Error, sync::Arc, time::SystemTime};

use log::info;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use warp::{
    self,
    filters::{body, header},
    http::{self, StatusCode},
    Filter,
};

use crate::color::Color;
use crate::config::Config;
use crate::dynamodb::DynamoDB;
use crate::image::Image;
use crate::mysql::MySQL;
use crate::s3::S3;

/*-------------------------------------*/

#[derive(Debug, Deserialize)]
struct Request {
    r: u8,
    g: u8,
    b: u8,
}

impl Request {
    fn new(json_string: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_string)
    }
}

/*-------------------------------------*/

#[derive(Serialize)]
struct Response {
    status: String,
    url: Option<String>,
}

impl Response {
    fn new(status: String, url: Option<String>) -> Self {
        Self { status, url }
    }

    fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }
}

/*-------------------------------------*/

fn create_filename() -> String {
    format!(
        "{}.png",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    )
}

async fn handler_logic(
    color: &Color,
    config: Arc<Config>,
    s3: Arc<Mutex<S3>>,
    rds: Arc<Mutex<MySQL>>,
    dynamodb: Arc<Mutex<DynamoDB>>,
) -> Result<String, Box<dyn Error>> {
    let image = Image::create_image(config.img_width, config.img_height, color);

    let filename = create_filename();

    let s3 = s3.lock().await;
    s3.upload(&filename, image).await?;
    let url = s3.get_presigned_url(&filename, config.s3.expiration_sec)?;

    rds.lock().await.insert(color)?;

    dynamodb.lock().await.insert(color).await?;

    Ok(url)
}

async fn handler(
    config: Arc<Config>,
    s3: Arc<Mutex<S3>>,
    rds: Arc<Mutex<MySQL>>,
    dynamodb: Arc<Mutex<DynamoDB>>,
    json_string: &str,
) -> http::Result<http::Response<String>> {
    let req = Request::new(json_string);

    if let Err(e) = req {
        info!("failed to parse json: {}", e);
        return http::Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(Response::new("error".to_string(), None).to_json_pretty());
    }

    let req = req.unwrap();
    let url = handler_logic(&Color::new(req.r, req.g, req.b), config, s3, rds, dynamodb).await;
    if let Err(e) = url {
        info!("aws operation failed: {}", e);
        return http::Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "application/json")
            .body(Response::new("error".to_string(), None).to_json_pretty());
    }

    http::Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Response::new("success".to_string(), Some(url.unwrap())).to_json_pretty())
}

/*-------------------------------------*/

pub async fn listen(config: &Arc<Config>) -> Result<(), Box<dyn Error>> {
    let s3 = Arc::new(Mutex::new(S3::new(&config.s3).await?));
    let rds = Arc::new(Mutex::new(MySQL::new(&config.rds)?));
    let dynamodb = Arc::new(Mutex::new(DynamoDB::new(&config.dynamodb).await?));

    let filter = warp::path!()
        .and(header::exact_ignore_case(
            "Content-Type",
            "application/json",
        ))
        .and(body::bytes())
        //ref: |https://stackoverflow.com/questions/66111599/how-can-i-achieve-shared-application-state-with-warp-async-routes|
        .and_then({
            let s3 = s3.clone();
            let rds = rds.clone();
            let dynamodb = dynamodb.clone();
            let config = config.clone();
            move |b: bytes::Bytes| {
                let s3 = s3.clone();
                let rds = rds.clone();
                let dynamodb = dynamodb.clone();
                let config = config.clone();
                async move {
                    let json_string = String::from_utf8(b.into_iter().collect()).unwrap();
                    handler(config, s3, rds, dynamodb, &json_string)
                        .await
                        .map_err(|_| warp::reject::reject())
                }
            }
        });

    let logger = warp::log::custom(|info| {
        println!();
        info!(
            "{} {} {} {} {}",
            info.remote_addr().unwrap(),
            info.method(),
            info.path(),
            info.status(),
            info.user_agent().unwrap(),
        );
    });

    warp::serve(filter.with(logger))
        .run(([127, 0, 0, 1], config.port))
        .await;

    Ok(())
}

/*-------------------------------------*/

#[cfg(test)]
mod handler_tests {

    use super::*;

    async fn f() -> Result<
        (
            Arc<Config>,
            Arc<Mutex<S3>>,
            Arc<Mutex<MySQL>>,
            Arc<Mutex<DynamoDB>>,
        ),
        Box<dyn Error>,
    > {
        let config = Arc::new(Config::new("./config.json"));
        let s3 = Arc::new(Mutex::new(S3::new(&config.s3).await?));
        let rds = Arc::new(Mutex::new(MySQL::new(&config.rds)?));
        let dynamodb = Arc::new(Mutex::new(DynamoDB::new(&config.dynamodb).await?));
        Ok((config, s3, rds, dynamodb))
    }

    #[tokio::test]
    async fn test01() -> Result<(), Box<dyn Error>> {
        let (config, s3, rds, dynamodb) = f().await?;

        let res = handler(config, s3, rds, dynamodb, "").await;
        println!("{:?}", res);
        assert!(res.is_ok());

        let res = res.unwrap();
        assert_eq!(StatusCode::BAD_REQUEST, res.status());
        assert_eq!(
            "{\n  \"status\": \"error\",\n  \"url\": null\n}",
            res.body()
        );

        Ok(())
    }

    #[tokio::test]
    async fn test02() -> Result<(), Box<dyn Error>> {
        let (config, s3, rds, dynamodb) = f().await?;

        let color = Color::new(100, 50, 25);

        let num_rds_row = rds.clone().lock().await.select_by_color(&color)?.len();
        let num_dynamodb_entry = dynamodb
            .clone()
            .lock()
            .await
            .select_by_color(&color)
            .await?
            .len();

        let res = handler(
            config,
            s3,
            rds.clone(),
            dynamodb.clone(),
            &format!(
                r#"{{"r": {}, "g": {}, "b": {}}}"#,
                color.r, color.g, color.b
            ),
        )
        .await;
        println!("{:?}", res);
        assert!(res.is_ok());

        let res = res.unwrap();
        assert_eq!(StatusCode::OK, res.status());
        assert!(res
            .body()
            .starts_with("{\n  \"status\": \"success\",\n  \"url\": \"https://"));

        assert_eq!(
            num_rds_row + 1,
            rds.lock().await.select_by_color(&color)?.len()
        );
        assert_eq!(
            num_dynamodb_entry + 1,
            dynamodb
                .clone()
                .lock()
                .await
                .select_by_color(&color)
                .await?
                .len(),
        );

        Ok(())
    }
}

/*-------------------------------------*/
