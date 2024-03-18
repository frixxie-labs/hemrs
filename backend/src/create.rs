use crate::db_connection_pool::DbConnectionPool;
use crate::db_connection_pool::Postgres;
use anyhow::Result;
use sensors::Dht11Entry;
use sqlx::PgPool;

pub trait Create<TPool>: Sized {
    type Connection: DbConnectionPool<TPool>;
    async fn create(self, connection: Self::Connection) -> Result<()>;
}

impl Create<PgPool> for Dht11Entry {
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
}
