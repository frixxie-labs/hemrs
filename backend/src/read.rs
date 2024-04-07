use anyhow::Result;
use sqlx::PgPool;

use crate::db_connection_pool::DbConnectionPool;

pub trait Read<TConnection: DbConnectionPool<PgPool>>: Sized {
    async fn read(connection: TConnection) -> Result<Self>;
}
