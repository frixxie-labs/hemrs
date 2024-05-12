use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::database::{
    insert::Insert,
    db_connection_pool::{DbConnectionPool, Postgres},
    delete::Delete,
    read::Read,
    update::Update,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sensor {
    name: String,
    unit: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Sensors {
    id: i32,
    name: String,
    unit: String,
}

impl Sensors {
    pub fn new(id: i32, name: String, unit: String) -> Self {
        Self { id, name, unit }
    }
}

impl Sensor {
    pub fn new(name: String, unit: String) -> Self {
        Self { name, unit }
    }
}

impl Read<Postgres> for Vec<Sensors> {
    async fn read(connection: Postgres) -> Result<Self> {
        let pool = connection.get_connection().await;
        let devices = sqlx::query_as::<_, Sensors>("SELECT id, name, unit FROM sensors")
            .fetch_all(&pool)
            .await?;
        Ok(devices)
    }
}

impl Insert<Postgres> for Sensor {
    async fn insert(self, connection: Postgres) -> Result<()> {
        let pool = connection.get_connection().await;
        sqlx::query("INSERT INTO sensors (name, unit) VALUES ($1, $2)")
            .bind(self.name)
            .bind(self.unit)
            .execute(&pool)
            .await?;
        Ok(())
    }
}

impl Delete<Postgres> for Sensors {
    async fn delete(self, connection: Postgres) -> Result<()> {
        let pool = connection.get_connection().await;
        sqlx::query("DELETE FROM sensors WHERE id = $1")
            .bind(self.id)
            .execute(&pool)
            .await?;
        Ok(())
    }
}

impl Update<Postgres> for Sensors {
    async fn update(self, connection: Postgres) -> anyhow::Result<()> {
        let pool = connection.get_connection().await;
        sqlx::query("UPDATE sensors SET name = $1,unit = $2 WHERE id = $3")
            .bind(self.name)
            .bind(self.unit)
            .bind(self.id)
            .execute(&pool)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use crate::{
        database::{
            insert::Insert, db_connection_pool::Postgres, delete::Delete, read::Read,
            update::Update,
        },
        sensors::{Sensor, Sensors},
    };

    #[sqlx::test]
    async fn insert(pool: PgPool) {
        let postgres = Postgres::new(pool);

        let sensor = Sensor::new("test".to_string(), "test".to_string());
        sensor.insert(postgres.clone()).await.unwrap();
        let sensors = Vec::<Sensors>::read(postgres.clone()).await.unwrap();
        assert!(!sensors.is_empty());
        assert_eq!(sensors.last().unwrap().name, "test");
        assert_eq!(sensors.last().unwrap().unit, "test");
    }

    #[sqlx::test]
    async fn delete(pool: PgPool) {
        let postgres = Postgres::new(pool);

        let sensor = Sensor::new("test".to_string(), "test".to_string());
        sensor.clone().insert(postgres.clone()).await.unwrap();
        let sensors = Vec::<Sensors>::read(postgres.clone()).await.unwrap();
        let sensor = sensors
            .last()
            .unwrap()
            .clone()
            .delete(postgres.clone())
            .await;
        assert!(sensor.is_ok());

        let sensors = Vec::<Sensors>::read(postgres.clone()).await.unwrap();
        assert!(sensors.len() <= 3);
    }

    #[sqlx::test]
    async fn update(pool: PgPool) {
        let postgres = Postgres::new(pool);

        let sensor = Sensor::new("test".to_string(), "test".to_string());
        sensor.clone().insert(postgres.clone()).await.unwrap();
        let sensors = Vec::<Sensors>::read(postgres.clone()).await.unwrap();
        let sensor = sensors.last().unwrap().clone();
        let sensor = Sensors::new(sensor.id, "test2".to_string(), "test2".to_string());
        sensor.clone().update(postgres.clone()).await.unwrap();

        let sensors = Vec::<Sensors>::read(postgres.clone()).await.unwrap();
        assert!(sensors.len() > 3);
        assert_eq!(sensors.last().unwrap().name, "test2");
        assert_eq!(sensors.last().unwrap().unit, "test2");
    }
}
