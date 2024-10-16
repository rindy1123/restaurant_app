#![warn(clippy::all)]

mod db;
mod server;

#[tokio::main]
async fn main() {
    let pool = db::get_connection_pool().await;
    db::run_migrations(&pool).await;
    server::run(pool).await;
}
