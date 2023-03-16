use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub s3: S3Config,
    pub rds: RDSConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct S3Config {
    pub bucket_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RDSConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database_name: String,
    pub table_name: String,
}

fn read_file(path: &str) -> String {
    std::fs::read_to_string(path).unwrap()
}

impl Config {
    pub fn new(config_file: &str) -> Self {
        serde_json::from_str(&read_file(config_file)).unwrap()
    }
}
