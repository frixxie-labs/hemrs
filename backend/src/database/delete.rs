use anyhow::Result;
use sqlx::PgPool;

use super::db_connection_pool::DbConnectionPool;

pub trait Delete<TConnection>
where
    Self: Sized,
    TConnection: DbConnectionPool<PgPool>,
{
    async fn delete(self, connection: TConnection) -> Result<()>;
}
