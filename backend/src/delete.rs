use crate::db_connection_pool::DbConnectionPool;
use anyhow::Result;
use sqlx::PgPool;

pub trait Delete<TConnection: DbConnectionPool<PgPool>>: Sized {
    async fn delete(self, connection: TConnection) -> Result<()>;
}
