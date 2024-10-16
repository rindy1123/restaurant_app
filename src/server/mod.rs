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
    // TODO: Get an address from environment variables
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
