use measurements::Measurement;
use metrics::histogram;
use metrics_exporter_prometheus::PrometheusBuilder;
use sqlx::PgPool;
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
        let measurements = Measurement::read_all_latest_measurements(pool)
            .await
            .unwrap();
        for measurement in measurements {
            let lables = [
                ("device_name", measurement.device_name),
                ("device_location", measurement.device_location),
                ("sensor_name", measurement.sensor_name),
                ("unit", measurement.unit),
            ];
            histogram!("measurements", &lables).record(measurement.value);
        }
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
    let connection = PgPool::connect(&opts.db_url).await.unwrap();

    let bg_pool = connection.clone();

    tokio::spawn(async move {
        bg_thread(&bg_pool).await;
    });

    let app = create_router(connection, metrics_handler);

    let listener = TcpListener::bind(&opts.host).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
