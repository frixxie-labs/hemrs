use anyhow::Result;
use sqlx::PgPool;

use super::db_connection_pool::DbConnectionPool;

pub trait Insert<TConnection>
where
    Self: Sized,
    TConnection: DbConnectionPool<PgPool>,
{
    async fn insert(self, connection: TConnection) -> Result<()>;
}
