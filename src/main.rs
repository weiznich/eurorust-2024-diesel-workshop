use clap::Parser;
use race_timing::service_config::Config;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() {
    let config = Config::parse();

    let subscriber = tracing_subscriber::Registry::default()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env());
    tracing::subscriber::set_global_default(subscriber).expect("Failed to setup tracing");

    let (router, _) = race_timing::setup(config.clone()).await;

    println!(
        "Starting server at http://{}:{}{}/index.html",
        config.address, config.port, config.base_url
    );
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.address, config.port))
        .await
        .expect("Failed to start server");

    axum::serve(listener, router).await.expect("App crashed");
}
