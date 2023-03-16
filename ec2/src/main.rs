use std::{error::Error, sync::Arc};

use ec2::config::Config;

const CONFIG_FILE: &str = "./config.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let config = Arc::new(Config::new(CONFIG_FILE));

    ec2::listen(&config).await
}
