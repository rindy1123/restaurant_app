use std::env;

use crate::db::ConnectionPool;
use axum::Router;

mod handler;

pub async fn run(pool: ConnectionPool) {
    let app = Router::new()
        .merge(handler::get_order_items())
        .merge(handler::get_order_item())
        .merge(handler::create_order())
        .merge(handler::delete_order_item())
        .with_state(pool);
    let host = env::var("API_HOST").unwrap();
    let port = env::var("API_PORT").unwrap();
    let address = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
