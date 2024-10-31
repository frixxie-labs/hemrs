use sqlx::PgPool;

pub trait DbConnectionPool<T> {
    async fn get_connection(&self) -> T;
}

#[derive(Clone, Debug)]
pub struct Postgres {
    pool: PgPool,
}

impl Postgres {
    pub fn new(pool: PgPool) -> Self {
        Postgres { pool }
    }
}

impl DbConnectionPool<PgPool> for Postgres {
    async fn get_connection(&self) -> PgPool {
        self.pool.clone()
    }
}
