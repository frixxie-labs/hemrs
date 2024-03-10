use axum::{
    routing::{get, post},
    Router,
};
use log::info;
use simple_logger::SimpleLogger;
use sqlx::PgPool;
use structopt::StructOpt;
use tokio::net::TcpListener;

use crate::handlers::{
    fetch_all_data, fetch_latest_data, fetch_mean_data, store_env_data, store_env_data_entry,
};

mod handlers;

#[derive(Debug, Clone, StructOpt)]
pub struct Opts {
    #[structopt(short, long, default_value = "0.0.0.0:65534")]
    host: String,

    #[structopt(
        short,
        long,
        env = "DATABASE_URL",
        default_value = "postgres://postgres:example@server:5432/postgres"
    )]
    db_url: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opts = Opts::from_args();
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()?;

    info!("Connecting to DB");
    let connection = PgPool::connect(&opts.db_url).await.unwrap();

    let app = Router::new()
        .route("/", get(fetch_all_data))
        .route("/latest", get(fetch_latest_data))
        .route("/mean", get(fetch_mean_data))
        .route("/", post(store_env_data))
        .route("/entry", post(store_env_data_entry))
        .with_state(connection);

    let listener = TcpListener::bind(&opts.host).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
