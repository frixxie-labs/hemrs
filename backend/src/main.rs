use axum::{
    routing::{get, post},
    Router,
};
use log::info;
use simple_logger::SimpleLogger;
use sqlx::PgPool;
use structopt::StructOpt;
use tokio::net::TcpListener;

use crate::handlers::{fetch_all_data, fetch_latest_data, store_env_data};

mod create;
mod db_connection_pool;
mod devices;
mod error;
mod handlers;
mod measurements;
mod read;
mod sensors;

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

    info!("Connecting to DB at {}", opts.db_url);
    let connection = PgPool::connect(&opts.db_url).await.unwrap();

    let pg_pool = db_connection_pool::Postgres::new(connection);

    let api = Router::new()
        .route("/all", get(fetch_all_data))
        .route("/latest", get(fetch_latest_data));

    let app = Router::new()
        .nest("/api", api)
        .route("/", post(store_env_data))
        .with_state(pg_pool);

    let listener = TcpListener::bind(&opts.host).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
