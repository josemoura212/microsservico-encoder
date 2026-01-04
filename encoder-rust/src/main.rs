#![allow(unused, dead_code)]

use tracing_subscriber::EnvFilter;

mod application;
mod domain;
mod framework;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logs();

    println!("Hello, world!");

    Ok(())
}

pub fn init_logs() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_test_writer()
        .try_init()
        .ok();
}
