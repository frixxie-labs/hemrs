use anyhow::Result;

use crate::db_connection_pool::DbConnectionPool;

pub trait Read<TPool>: Sized {
    type Connection: DbConnectionPool<TPool>;
    async fn read(connection: Self::Connection) -> Result<Self>;
}
