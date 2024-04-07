use crate::db_connection_pool::DbConnectionPool;
use anyhow::Result;
use sqlx::PgPool;

pub trait Create<TConnection: DbConnectionPool<PgPool>>: Sized {
    async fn create(self, connection: TConnection) -> Result<()>;
}
