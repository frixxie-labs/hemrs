use devices::Device;
use measurements::Measurement;
use metrics::{counter, gauge};
use metrics_exporter_prometheus::PrometheusBuilder;
use moka::future::Cache;
use sensors::Sensor;
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

async fn bg_thread(pool: &PgPool, cache: &Cache<(i32, i32), Measurement>) {
    loop {
        debug!("Running background thread");
        let devices = Device::read(pool).await.unwrap();
        let mut device_sensors: Vec<(Device, Sensor)> = Vec::new();
        for device in devices {
            let sensors = Sensor::read_by_device_id(pool, device.id).await.unwrap();
            for sensor in sensors {
                device_sensors.push((device.clone(), sensor));
            }
        }

        let now = chrono::Utc::now();
        for (device, sensor) in device_sensors {
            //check cache first
            if let Some(measurement) = cache.get(&(device.id, sensor.id)).await {
                if measurement.timestamp >= now - chrono::Duration::seconds(300) {
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
            } else {
                // If not in cache, read from DB
                let measurement =
                    Measurement::read_latest_by_device_id_and_sensor_id(device.id, sensor.id, pool)
                        .await
                        .unwrap();
                if measurement.timestamp >= now - chrono::Duration::seconds(300) {
                    let lables = [
                        ("device_name", measurement.device_name.clone()),
                        ("device_location", measurement.device_location.clone()),
                        ("sensor_name", measurement.sensor_name.clone()),
                        ("unit", measurement.unit.clone()),
                    ];
                    gauge!("measurements", &lables).set(measurement.value);
                    // Store in cache
                    cache
                        .insert((device.id, sensor.id), measurement.clone())
                        .await;
                }
            }
        }
        counter!("hemrs_pg_pool_size").absolute(pool.size() as u64);
        counter!("hemrs_cache_size").absolute(cache.entry_count());
        debug!("Background thread finished");
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
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

    let measurement_cache: Cache<(i32, i32), Measurement> = Cache::builder()
        .max_capacity(128)
        .time_to_live(std::time::Duration::from_secs(60))
        .build();

    let bg_pool = connection.clone();
    let measurement_cache_bg = measurement_cache.clone();

    tokio::spawn(async move {
        bg_thread(&bg_pool, &measurement_cache_bg).await;
    });

    let app = create_router(connection, metrics_handler, measurement_cache);

    let listener = TcpListener::bind(&opts.host).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
