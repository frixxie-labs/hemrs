use measurements::Measurement;
use metrics_exporter_prometheus::PrometheusBuilder;
use moka::future::Cache;
use sqlx::postgres::PgPoolOptions;
use structopt::StructOpt;
use tokio::{net::TcpListener, sync::mpsc::channel};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::{
    background_tasks::{handle_insert_measurement_bg_thread, refresh_views, update_metrics},
    handlers::create_router,
    measurements::NewMeasurement,
};

mod background_tasks;
mod devices;
mod handlers;
mod measurements;
mod sensors;

#[derive(Debug, Clone)]
enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl std::str::FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "trace" => Ok(LogLevel::Trace),
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            _ => Err("unknown log level".to_string()),
        }
    }
}

#[derive(Debug, Clone, StructOpt)]
pub struct Opts {
    #[structopt(short, long, default_value = "0.0.0.0:65534")]
    host: String,

    #[structopt(
        short,
        long,
        env = "DATABASE_URL",
        default_value = "postgres://postgres:example@localhost:5432/postgres"
    )]
    db_url: String,

    #[structopt(short, long, default_value = "info")]
    log_level: LogLevel,
}

impl From<LogLevel> for Level {
    fn from(log_level: LogLevel) -> Self {
        match log_level {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opts = Opts::from_args();
    let level: Level = opts.log_level.into();
    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .json()
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();
    let metrics_handler = PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install recorder/exporter");

    info!("Connecting to DB at {}", opts.db_url);
    let connection = PgPoolOptions::new().connect(&opts.db_url).await.unwrap();

    let measurement_cache: Cache<(i32, i32), Measurement> = Cache::builder()
        .max_capacity(128)
        .time_to_live(std::time::Duration::from_secs(60))
        .build();

    let bg_pool = connection.clone();
    let measurement_cache_bg = measurement_cache.clone();

    tokio::spawn(async move {
        update_metrics(&bg_pool, &measurement_cache_bg).await;
    });

    let (tx, rx) = channel::<NewMeasurement>(1 << 13);

    let insert_pool = connection.clone();
    let insert_cache = measurement_cache.clone();

    tokio::spawn(async move {
        handle_insert_measurement_bg_thread(rx, insert_pool, insert_cache).await;
    });

    let refresh_pool = connection.clone();

    tokio::spawn(async move {
        refresh_views(&refresh_pool).await.unwrap();
    });

    let app = create_router(connection, metrics_handler, measurement_cache, tx);

    let listener = TcpListener::bind(&opts.host).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
