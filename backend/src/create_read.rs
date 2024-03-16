use anyhow::Result;
use sensors::Dht11Entry;
use sqlx::PgPool;

use crate::db_connection_pool::DbConnectionPool;
pub trait CreateRead<TPool>: Sized {
    async fn create(self, connection_pool: impl DbConnectionPool<TPool>) -> Result<()>;
    async fn read(connection_pool: impl DbConnectionPool<TPool>) -> Result<Self>;
    async fn read_all(connection_pool: impl DbConnectionPool<TPool>) -> Result<Vec<Self>>;
}

impl CreateRead<PgPool> for Dht11Entry {
    async fn create(self, connection_pool: impl DbConnectionPool<PgPool>) -> Result<()> {
        let pool = connection_pool.get_connection().await;
        sqlx::query("INSERT INTO env_data VALUES ($1, $2, $3, $4)")
            .bind(self.ts)
            .bind(self.room)
            .bind(self.temperature)
            .bind(self.humidity)
            .execute(&pool)
            .await?;
        Ok(())
    }

    async fn read(connection_pool: impl DbConnectionPool<PgPool>) -> Result<Self> {
        let pool = connection_pool.get_connection().await;
        let env_data_entry =
            sqlx::query_as::<_, Dht11Entry>("SELECT * FROM env_data ORDER BY ts DESC LIMIT 1")
                .fetch_one(&pool)
                .await?;
        Ok(env_data_entry)
    }

    async fn read_all(connection_pool: impl DbConnectionPool<PgPool>) -> Result<Vec<Self>> {
        let pool = connection_pool.get_connection().await;
        let env_data_entries =
            sqlx::query_as::<_, Dht11Entry>("SELECT * FROM env_data ORDER BY ts DESC")
                .fetch_all(&pool)
                .await?;
        Ok(env_data_entries)
    }
}
