use sqlx::PgPool;
use structopt::StructOpt;
use tokio::net::TcpListener;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::handlers::create_router;

mod database;
mod devices;
mod handlers;
mod measurements;
mod sensors;

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

    #[structopt(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opts = Opts::from_args();
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .json()
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();

    info!("Connecting to DB at {}", opts.db_url);
    let connection = PgPool::connect(&opts.db_url).await.unwrap();

    let app = create_router(connection);

    let listener = TcpListener::bind(&opts.host).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
