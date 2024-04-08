use anyhow::Result;

use super::db_connection_pool::DbConnectionPool;


pub trait Query<TPool, TQuery, TResult> {
    type Connection: DbConnectionPool<TPool>;
    async fn query(connection: Self::Connection, query: TQuery) -> Result<TResult>;
}
