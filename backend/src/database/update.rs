use anyhow::Result;
use sqlx::PgPool;

use super::db_connection_pool::DbConnectionPool;

pub trait Update<TConnection>
where
    Self: Sized,
    TConnection: DbConnectionPool<PgPool> + Clone,
{
    async fn update(self, connection: TConnection) -> Result<()>;
}
