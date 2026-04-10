use clap::Parser;
use measurements::Measurement;
use metrics_exporter_prometheus::PrometheusBuilder;
use moka::future::Cache;
use sqlx::postgres::PgPoolOptions;
use tokio::{net::TcpListener, sync::mpsc::channel};
use tracing::{error, info, Level};
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

/// Command line options for the IoT sensor monitoring backend server.
///
/// This struct defines configuration options that can be provided via command line
/// arguments or environment variables to customize the server's behavior.
#[derive(Debug, Clone, Parser)]
pub struct Opts {
    /// Host address and port to bind the HTTP server to.
    ///
    /// Defaults to "0.0.0.0:65534" to listen on all interfaces on port 65534.
    #[arg(short, long, default_value = "0.0.0.0:65534")]
    host: String,

    /// PostgreSQL database connection URL.
    ///
    /// Can be provided via the DATABASE_URL environment variable or the -d flag.
    /// Defaults to a local PostgreSQL instance.
    #[arg(
        short,
        long,
        env = "DATABASE_URL",
        default_value = "postgres://postgres:example@localhost:5432/postgres"
    )]
    db_url: String,

    /// Logging level for the application.
    ///
    /// Valid values are: trace, debug, info, warn, error.
    /// Defaults to "info" level.
    #[arg(short, long, default_value = "info")]
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

#[cfg(test)]
mod tests {
    use super::LogLevel;
    use std::str::FromStr;
    use tracing::Level;

    const VALID_LEVELS: &[&str] = &["trace", "debug", "info", "warn", "error"];

    #[test]
    fn valid_log_levels_parse_successfully() {
        for &s in VALID_LEVELS {
            assert!(
                LogLevel::from_str(s).is_ok(),
                "expected '{s}' to parse as a valid LogLevel"
            );
        }
    }

    #[quickcheck_macros::quickcheck]
    fn unknown_log_level_returns_err(s: String) -> bool {
        if VALID_LEVELS.contains(&s.as_str()) {
            return true; // skip known-good values
        }
        LogLevel::from_str(&s).is_err()
    }

    #[quickcheck_macros::quickcheck]
    fn log_level_from_is_total(raw: u8) -> bool {
        // Map the 5 variants by index; From<LogLevel> must never panic.
        let level = match raw % 5 {
            0 => LogLevel::Trace,
            1 => LogLevel::Debug,
            2 => LogLevel::Info,
            3 => LogLevel::Warn,
            _ => LogLevel::Error,
        };
        let tracing_level: Level = level.into();
        // The conversion is exhaustive — just confirm it produces a valid Level.
        matches!(
            tracing_level,
            Level::TRACE | Level::DEBUG | Level::INFO | Level::WARN | Level::ERROR
        )
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opts = Opts::parse();
    let level: Level = opts.log_level.into();
    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();

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

    let (tx, rx) = channel::<NewMeasurement>(1 << 13);

    let insert_pool = connection.clone();
    let insert_cache = measurement_cache.clone();

    let refresh_pool = connection.clone();

    let app = create_router(connection, metrics_handler, measurement_cache, tx);

    let listener = TcpListener::bind(&opts.host).await.unwrap();

    tokio::select! {
        _ = update_metrics(&bg_pool, &measurement_cache_bg) => {
            error!("update_metrics task exited unexpectedly");
        }
        _ = handle_insert_measurement_bg_thread(rx, insert_pool, insert_cache) => {
            error!("insert_measurement background task exited unexpectedly");
        }
        result = refresh_views(&refresh_pool) => {
            match result {
                Ok(()) => error!("refresh_views task exited unexpectedly"),
                Err(e) => error!("refresh_views task failed: {e:#}"),
            }
        }
        result = axum::serve(listener, app) => {
            match result {
                Ok(()) => info!("server shut down gracefully"),
                Err(e) => error!("server error: {e:#}"),
            }
        }
    }

    Ok(())
}
