use axum::{
    routing::{get, post},
    Router,
};
use handlers::{
    fetch_all_measurements, fetch_devices, fetch_latest_measurement, fetch_sensors,
    store_measurements,
};
use log::info;
use simple_logger::SimpleLogger;
use sqlx::PgPool;
use structopt::StructOpt;
use tokio::net::TcpListener;

mod create;
mod db_connection_pool;
mod devices;
mod error;
mod handlers;
mod measurements;
mod query;
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

    let measurements = Router::new()
        .route("/measurements", get(fetch_all_measurements))
        .route("/measurements/latest", get(fetch_latest_measurement));

    let devices = Router::new().route("/devices", get(fetch_devices));

    let sensors = Router::new().route("/sensors", get(fetch_sensors));

    let api = Router::new()
        .nest("/api", measurements)
        .nest("/api", devices)
        .nest("/api", sensors);

    let app = Router::new()
        .nest("/api", api)
        .route("/", post(store_measurements))
        .with_state(pg_pool);

    let listener = TcpListener::bind(&opts.host).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
