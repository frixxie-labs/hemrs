use anyhow::Result;
use sensors::Dht11Entry;
use sqlx::PgPool;

use crate::db_connection_pool::{DbConnectionPool, Postgres};
pub trait Read<TPool>: Sized {
    type Connection: DbConnectionPool<TPool>;
    async fn read(connection: Self::Connection) -> Result<Self>;
}

impl Read<PgPool> for Dht11Entry {
    type Connection = Postgres;

    async fn read(connection: Self::Connection) -> Result<Self> {
        let pool = connection.get_connection().await;
        let dht11_entry =
            sqlx::query_as::<_, Dht11Entry>("SELECT * FROM env_data ORDER BY ts DESC LIMIT 1")
                .fetch_one(&pool)
                .await?;
        Ok(dht11_entry)
    }
}

impl Read<PgPool> for Vec<Dht11Entry> {
    type Connection = Postgres;

    async fn read(connection: Self::Connection) -> Result<Self> {
        let pool = connection.get_connection().await;
        let dht11_entries =
            sqlx::query_as::<_, Dht11Entry>("SELECT * FROM env_data ORDER BY ts DESC")
                .fetch_all(&pool)
                .await?;
        Ok(dht11_entries)
    }
}
