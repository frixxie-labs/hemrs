use anyhow::Result;
use axum::{extract::State, Json};
use log::info;

use sensors::{Dht11, Dht11Entry};

use crate::db_connection_pool::Postgres;

use crate::create_read::CreateRead;
use crate::error::HandlerError;

pub async fn store_env_data(
    State(pg_pool): State<Postgres>,
    Json(env_data): Json<Dht11>,
) -> Result<String, HandlerError> {
    info!("POST /");
    let env_data_entry: Dht11Entry = env_data.into();

    env_data_entry
        .create(pg_pool)
        .await
        .map_err(|e| HandlerError::new(500, format!("Failed to store data in database: {}", e)))?;

    Ok("OK".to_string())
}

pub async fn fetch_mean_data(
    State(pg_pool): State<Postgres>,
) -> Result<Json<Dht11Entry>, HandlerError> {
    info!("GET api/mean");
    let rows = Dht11Entry::read_all(pg_pool).await.map_err(|e| {
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;

    let (sum_temperature, sum_humidity) = rows.iter().fold((0.0, 0.0), |(temp, hum), row| {
        (temp + row.temperature, hum + row.humidity)
    });
    let temperature = sum_temperature / rows.len() as f32;
    let humidity = sum_humidity / rows.len() as f32;

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
    let rows = Dht11Entry::read_all(pool).await.map_err(|e| {
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;

    Ok(Json(rows))
}
