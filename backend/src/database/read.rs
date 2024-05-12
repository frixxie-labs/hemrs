use anyhow::Result;
use sqlx::PgPool;

use super::db_connection_pool::DbConnectionPool;

pub trait Read<TConnection>: Sized
where
    TConnection: DbConnectionPool<PgPool>,
{
    async fn read(connection: TConnection) -> Result<Self>;
}
