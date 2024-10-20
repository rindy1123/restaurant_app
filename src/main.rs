#![warn(clippy::all)]

mod client;
mod db;
mod server;

#[tokio::main]
async fn main() {
    // Initialize the connection pool and run migrations
    let pool = db::get_connection_pool().await;
    db::run_migrations(&pool).await;

    // 10 clients calling the server at the same time
    for _ in 0..10 {
        tokio::spawn(async {
            client::mock_requests().await;
        });
    }

    // Start the server
    server::run(pool).await;
}
