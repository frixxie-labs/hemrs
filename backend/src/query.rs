use anyhow::Result;

use crate::db_connection_pool::DbConnectionPool;

pub trait Query<TPool, TQuery, TResult> {
    type Connection: DbConnectionPool<TPool>;
    async fn query(connection: Self::Connection, query: TQuery) -> Result<TResult>;
}
