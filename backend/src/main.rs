use log::info;
use simple_logger::SimpleLogger;
use sqlx::PgPool;
use structopt::StructOpt;
use tokio::net::TcpListener;

use crate::handlers::create_router;

mod create;
mod db_connection_pool;
mod delete;
mod devices;
mod error;
mod handlers;
mod measurements;
mod query;
mod read;
mod sensors;
mod update;

#[derive(Debug, Clone, StructOpt)]
pub struct Opts {
    #[structopt(short, long, default_value = "0.0.0.0:65534")]
    host: String,

    #[structopt(
        short,
        long,
        env = "DATABASE_URL",
        default_value = "postgres://postgres:example@server:5432/postgres"
    )]
    db_url: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opts = Opts::from_args();
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()?;

    info!("Connecting to DB at {}", opts.db_url);
    let connection = PgPool::connect(&opts.db_url).await.unwrap();

    let app = create_router(connection);

    let listener = TcpListener::bind(&opts.host).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
