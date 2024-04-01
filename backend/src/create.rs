use crate::db_connection_pool::DbConnectionPool;
use anyhow::Result;

pub trait Create<TPool>: Sized {
    type Connection: DbConnectionPool<TPool>;
    async fn create(self, connection: Self::Connection) -> Result<()>;
}
