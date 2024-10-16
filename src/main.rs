#![warn(clippy::all)]

mod db;
mod server;

#[tokio::main]
async fn main() {
    // Initialize the connection pool and run migrations
    let pool = db::get_connection_pool().await;
    db::run_migrations(&pool).await;

    // Start the server
    server::run(pool).await;
}
