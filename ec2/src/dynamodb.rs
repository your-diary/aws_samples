#[cfg(test)]
use std::collections::HashMap;
use std::error::Error;
use std::time::SystemTime;

use aws_sdk_dynamodb::model::AttributeValue;
#[cfg(test)]
use tokio_stream::StreamExt;

use super::color::Color;
use super::config::DynamoDBConfig;

pub struct DynamoDB {
    table_name: String,
    client: aws_sdk_dynamodb::Client,
}

impl DynamoDB {
    pub async fn new(dynamodb_config: &DynamoDBConfig) -> Result<Self, Box<dyn Error>> {
        let config = aws_config::load_from_env().await;
        let client = aws_sdk_dynamodb::Client::new(&config);

        Ok(Self {
            table_name: dynamodb_config.table_name.clone(),
            client,
        })
    }

    pub async fn insert(&self, color: &Color) -> Result<(), Box<dyn Error>> {
        let timestamp = AttributeValue::S(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .to_string(),
        );
        let r = AttributeValue::N(color.r.to_string());
        let g = AttributeValue::N(color.g.to_string());
        let b = AttributeValue::N(color.b.to_string());

        let request = self
            .client
            .put_item()
            .table_name(&self.table_name)
            .item("timestamp", timestamp)
            .item("r", r)
            .item("g", g)
            .item("b", b);

        request.send().await?;
        Ok(())
    }

    #[cfg(test)]
    pub async fn select_by_color(
        &self,
        color: &Color,
    ) -> Result<Vec<HashMap<String, AttributeValue>>, Box<dyn Error>> {
        self.client
            .scan()
            .table_name(&self.table_name)
            .filter_expression("r = :r and g = :g and b = :b")
            .expression_attribute_values(":r", AttributeValue::N(color.r.to_string()))
            .expression_attribute_values(":g", AttributeValue::N(color.g.to_string()))
            .expression_attribute_values(":b", AttributeValue::N(color.b.to_string()))
            .into_paginator()
            .items()
            .send()
            .collect::<Result<Vec<_>, _>>()
            .await
            .map_err(|e| e.into())
    }
}

/*-------------------------------------*/

#[cfg(test)]
mod dynamodb_tests {

    use super::super::config::Config;
    use super::*;

    #[tokio::test]
    async fn test01() -> Result<(), Box<dyn Error>> {
        let config = Config::new("./config.json");
        let dynamodb = DynamoDB::new(&config.dynamodb).await?;
        let color = Color::new(100, 50, 20);

        let num_entry = dynamodb.select_by_color(&color).await?.len();

        let res = dynamodb.insert(&color).await;
        println!("{:?}", res);
        assert!(res.is_ok());

        assert_eq!(num_entry + 1, dynamodb.select_by_color(&color).await?.len());

        let res = dynamodb.select_by_color(&color).await;
        println!("{:?}", res);
        assert!(res.is_ok());

        Ok(())
    }
}

/*-------------------------------------*/
