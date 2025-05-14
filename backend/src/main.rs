use measurements::Measurement;
use metrics::{counter, gauge};
use metrics_exporter_prometheus::PrometheusBuilder;
use sqlx::{postgres::PgPoolOptions, PgPool};
use structopt::StructOpt;
use tokio::net::TcpListener;
use tracing::{debug, info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::handlers::create_router;

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

async fn bg_thread(pool: &PgPool) {
    loop {
        debug!("Running background thread");
        let now = chrono::Utc::now();
        let measurements = Measurement::read_all_latest_measurements(pool)
            .await
            .unwrap();
        for measurement in measurements {
            if measurement.timestamp < now - chrono::Duration::seconds(300) {
                continue;
            }
            let lables = [
                ("device_name", measurement.device_name),
                ("device_location", measurement.device_location),
                ("sensor_name", measurement.sensor_name),
                ("unit", measurement.unit),
            ];
            gauge!("measurements", &lables).set(measurement.value);
        }
        counter!("PgPoolSize").absolute(pool.size() as u64);
        debug!("Background thread finished");
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
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
    let connection = PgPoolOptions::new()
        .max_connections(100)
        .min_connections(8)
        .idle_timeout(std::time::Duration::from_secs(30))
        .connect(&opts.db_url)
        .await
        .unwrap();

    let bg_pool = connection.clone();

    tokio::spawn(async move {
        bg_thread(&bg_pool).await;
    });

    let app = create_router(connection, metrics_handler);

    let listener = TcpListener::bind(&opts.host).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
