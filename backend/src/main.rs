use axum::{
    routing::{get, post},
    Router, Server,
};
use log::info;
use simple_logger::SimpleLogger;
use sqlx::PgPool;
use structopt::StructOpt;

use crate::handlers::{fetch_all_data, store_env_data, store_env_data_entry};

mod handlers;

#[derive(Debug, Clone, StructOpt)]
pub struct Opts {
    #[structopt(short, long, default_value = "0.0.0.0:65534")]
    host: String,

    #[structopt(
        short,
        long,
        env = "DATABASE_URL",
        default_value = "postgres://postgres:password@localhost:5432/env_data"
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
        .route("/", post(store_env_data))
        .route("/entry", post(store_env_data_entry))
        .with_state(connection);

    Server::bind(&opts.host.parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
