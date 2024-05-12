use anyhow::Result;
use sqlx::PgPool;

use super::db_connection_pool::DbConnectionPool;

pub trait Query<TConnection: DbConnectionPool<PgPool>, TResult> {
    async fn query(self: Self, connection: TConnection) -> Result<TResult>;
}
