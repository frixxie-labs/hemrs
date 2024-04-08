use crate::{create::Create, db_connection_pool::DbConnectionPool, delete::Delete};
use anyhow::Result;
use sqlx::PgPool;

pub trait Update<TConnection>
where
    Self: Sized + Create<TConnection> + Delete<TConnection> + Clone,
    TConnection: DbConnectionPool<PgPool> + Clone,
{
    async fn update(self, connection: TConnection) -> Result<()> {
        self.clone().delete(connection.clone()).await?;
        self.create(connection).await?;
        Ok(())
    }
}
