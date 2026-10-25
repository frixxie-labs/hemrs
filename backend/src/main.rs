use anyhow::Context;
use clap::Parser;
use metrics_exporter_prometheus::PrometheusBuilder;
use sqlx::postgres::PgPoolOptions;
use tokio::{net::TcpListener, sync::mpsc::channel};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

use crate::{
    background_tasks::{handle_insert_measurement_bg_thread, refresh_views, update_metrics},
    handlers::create_router,
    measurements::{NewMeasurement, latest::LatestMeasurements},
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
    #[arg(short = 'H', long, default_value = "0.0.0.0:65534")]
    host: String,

    /// PostgreSQL database connection URL.
    ///
    /// Can be provided via the DATABASE_URL environment variable or the -d flag.
    /// Defaults to a local PostgreSQL instance.
    #[arg(
        short,
        long,
        env = "DATABASE_URL",
        default_value = "postgres://postgres:admin@localhost:5432/postgres"
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

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opts = Opts::parse();
    let level: Level = opts.log_level.into();
    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .json()
        .flatten_event(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();
    let metrics_handler = PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install recorder/exporter");

    info!("Connecting to database");
    let connection = PgPoolOptions::new()
        .connect(&opts.db_url)
        .await
        .context("Connecting to database")?;

    let measurement_cache = LatestMeasurements::new(512, std::time::Duration::from_secs(60));

    let bg_pool = connection.clone();
    let measurement_cache_bg = measurement_cache.clone();

    let (tx, rx) = channel::<NewMeasurement>(1 << 13);
    let (measurement_updates, _) = tokio::sync::broadcast::channel(1024);

    let insert_pool = connection.clone();
    let insert_cache = measurement_cache.clone();

    let refresh_pool = connection.clone();

    let app = create_router(
        connection,
        metrics_handler,
        measurement_cache,
        tx,
        measurement_updates.clone(),
    );

    let listener = TcpListener::bind(&opts.host)
        .await
        .context("Binding HTTP listener")?;
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let mut server = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await
    });
    let mut worker = tokio::spawn(handle_insert_measurement_bg_thread(
        rx,
        insert_pool,
        insert_cache,
        measurement_updates,
    ));

    tokio::select! {
        result = shutdown_signal() => {
            result?;
            info!("Shutting down HTTP server and draining accepted measurements");
        }
        _ = update_metrics(&bg_pool, &measurement_cache_bg) => {
            anyhow::bail!("Metrics task exited unexpectedly");
        }
        result = &mut worker => {
            result?;
            anyhow::bail!("Measurement worker exited unexpectedly");
        }
        _ = refresh_views(&refresh_pool) => {
            anyhow::bail!("View refresh task exited unexpectedly");
        }
        result = &mut server => {
            result??;
            anyhow::bail!("HTTP server exited unexpectedly");
        }
    }

    let _ = shutdown_tx.send(());
    tokio::time::timeout(std::time::Duration::from_secs(30), async {
        server.await??;
        worker.await?;
        Ok::<(), anyhow::Error>(())
    })
    .await
    .context("Timed out draining HTTP connections and measurement queue")??;
    Ok(())
}

async fn shutdown_signal() -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
        tokio::select! {
            result = tokio::signal::ctrl_c() => result?,
            _ = terminate.recv() => {},
        }
    }
    #[cfg(not(unix))]
    tokio::signal::ctrl_c().await?;
    Ok(())
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
