pub mod color;
pub mod config;
pub mod image;
pub mod mysql;
pub mod s3;

use std::{error::Error, sync::Arc};

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

async fn f(
    color: &Color,
    config: Arc<Config>,
    s3: Arc<Mutex<S3>>,
    rds: Arc<Mutex<MySQL>>,
) -> Result<String, Box<dyn Error>> {
    //TODO
    //1. `f`という名前を変える
    //2. `width`などをconfigに外だし
    //3. `rds`を使用
    //4. テストを書く(`f`ではなく`handler`のテストで良い気もする)

    let width = 200;
    let height = 100;
    let image = Image::create_image(width, height, color);

    let filename = S3::create_filename();

    let s3 = s3.lock().await;
    s3.upload(&filename, image).await?;
    let url = s3.get_presigned_url(&filename, 30)?;

    Ok(url)
}

async fn handler(
    config: Arc<Config>,
    json_string: String,
    s3: Arc<Mutex<S3>>,
    rds: Arc<Mutex<MySQL>>,
) -> http::Result<http::Response<String>> {
    let req = Request::new(&json_string);

    if let Err(e) = req {
        info!("failed to parse json: {}", e);
        return http::Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(Response::new("error".to_string(), None).to_json_pretty());
    }

    let req = req.unwrap();
    let url = f(&Color::new(req.r, req.g, req.b), config, s3, rds).await;
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
            let config = config.clone();
            move |b: bytes::Bytes| {
                let s3 = s3.clone();
                let rds = rds.clone();
                let config = config.clone();
                async move {
                    let s = String::from_utf8(b.into_iter().collect()).unwrap();
                    handler(config, s, s3, rds)
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
