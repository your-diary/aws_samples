use std::{error::Error, time::SystemTime};

use lambda_runtime::LambdaEvent;
use serde::{Deserialize, Serialize};

/*-------------------------------------*/

#[derive(Deserialize)]
struct Request {
    content: Option<String>,
}

impl Request {
    fn new(json_string: &str) -> Self {
        serde_json::from_str(json_string).unwrap()
    }
}

/*-------------------------------------*/

#[derive(Debug, Serialize, PartialEq)]
struct Response {
    status: String,
    filename: Option<String>,
}

impl Response {
    fn new(status: String, filename: Option<String>) -> Self {
        Self { status, filename }
    }
}

impl std::fmt::Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Response {{ status: {}, filename: {:?} }}",
            self.status, self.filename
        )
    }
}

impl Error for Response {}

/*-------------------------------------*/

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    lambda_runtime::run(lambda_runtime::service_fn(handler)).await?;
    Ok(())
}

const BUCKET_NAME: &str = "bucket-test-001-a";

async fn handler(req: LambdaEvent<serde_json::Value>) -> Result<Response, Response> {
    let req = Request::new(&serde_json::to_string(&req.payload).unwrap());
    if (req.content.is_none()) {
        return Err(Response::new("error".to_string(), None));
    }

    let config = aws_config::load_from_env().await;
    let s3_client = aws_sdk_s3::Client::new(&config);

    let filename = format!(
        "{}.txt",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let res = s3_client
        .put_object()
        .bucket(BUCKET_NAME)
        .body(req.content.unwrap().as_bytes().to_owned().into())
        .key(&filename)
        .content_type("text/plain")
        .send()
        .await;
    if let Err(e) = res {
        println!("{}", e);
        Err(Response::new("error".to_string(), None))
    } else {
        Ok(Response::new("success".to_string(), Some(filename)))
    }
}

/*-------------------------------------*/

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test01() {
        let input = serde_json::from_str(r#"{"content": "hello"}"#).unwrap();
        let context = lambda_runtime::Context::default();

        let event = lambda_runtime::LambdaEvent::new(input, context);

        let res = handler(event).await;
        println!("{:?}", res);
        assert!(res.is_ok());
        assert_eq!("success", res.unwrap().status);
    }
}

/*-------------------------------------*/
