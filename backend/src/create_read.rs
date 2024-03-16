use anyhow::Result;
use sensors::Dht11Entry;
use sqlx::PgPool;

use crate::db_connection_pool::{DbConnectionPool, Postgres};
pub trait CreateRead<TPool>: Sized {
    type Connection: DbConnectionPool<TPool>;

    async fn create(self, connection: Self::Connection) -> Result<()>;
    async fn read(connection: Self::Connection) -> Result<Self>;
    async fn read_all(connection: Self::Connection) -> Result<Vec<Self>>;
}

impl CreateRead<PgPool> for Dht11Entry {
    type Connection = Postgres;

    async fn create(self, connection: Self::Connection) -> Result<()> {
        let pool = connection.get_connection().await;
        sqlx::query("INSERT INTO env_data VALUES ($1, $2, $3, $4)")
            .bind(self.ts)
            .bind(self.room)
            .bind(self.temperature)
            .bind(self.humidity)
            .execute(&pool)
            .await?;
        Ok(())
    }

    async fn read(connection: Self::Connection) -> Result<Self> {
        let pool = connection.get_connection().await;
        let dht11_entry =
            sqlx::query_as::<_, Dht11Entry>("SELECT * FROM env_data ORDER BY ts DESC LIMIT 1")
                .fetch_one(&pool)
                .await?;
        Ok(dht11_entry)
    }

    async fn read_all(connection: Self::Connection) -> Result<Vec<Self>> {
        let pool = connection.get_connection().await;
        let dht11_entries =
            sqlx::query_as::<_, Dht11Entry>("SELECT * FROM env_data ORDER BY ts DESC")
                .fetch_all(&pool)
                .await?;
        Ok(dht11_entries)
    }
}
