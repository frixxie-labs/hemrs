use anyhow::Result;
use axum::{extract::State, Json};
use log::info;

use sensors::{Dht11, Dht11Entry};

use crate::create::Create;
use crate::db_connection_pool::Postgres;

use crate::error::HandlerError;
use crate::read::Read;

pub async fn store_env_data(
    State(pg_pool): State<Postgres>,
    Json(dht11_data): Json<Dht11>,
) -> Result<String, HandlerError> {
    info!("POST /");
    let dht11_entry: Dht11Entry = dht11_data.into();

    dht11_entry
        .create(pg_pool)
        .await
        .map_err(|e| HandlerError::new(500, format!("Failed to store data in database: {}", e)))?;

    Ok("OK".to_string())
}

pub async fn fetch_mean_data(
    State(pg_pool): State<Postgres>,
) -> Result<Json<Dht11Entry>, HandlerError> {
    info!("GET api/mean");
    let dht11_entries = Vec::<Dht11Entry>::read(pg_pool).await.map_err(|e| {
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;

    let (sum_temperature, sum_humidity) =
        dht11_entries.iter().fold((0.0, 0.0), |(temp, hum), row| {
            (temp + row.temperature, hum + row.humidity)
        });
    let temperature = sum_temperature / dht11_entries.len() as f32;
    let humidity = sum_humidity / dht11_entries.len() as f32;

    let result = Dht11Entry::new(
        chrono::Utc::now(),
        "mean".to_string(),
        temperature,
        humidity,
    );

    Ok(Json(result))
}

pub async fn fetch_latest_data(
    State(pg_pool): State<Postgres>,
) -> Result<Json<Dht11Entry>, HandlerError> {
    info!("GET api/latest");
    let entry = Dht11Entry::read(pg_pool).await.map_err(|e| {
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;

    Ok(Json(entry))
}

pub async fn fetch_all_data(
    State(pool): State<Postgres>,
) -> Result<Json<Vec<Dht11Entry>>, HandlerError> {
    info!("GET api/all");
    let entry = Vec::<Dht11Entry>::read(pool).await.map_err(|e| {
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;

    Ok(Json(entry))
}
