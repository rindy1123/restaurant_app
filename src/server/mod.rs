use crate::db::ConnectionPool;
use axum::{
    routing::{delete, get, post},
    Router,
};

mod handler;

pub async fn run(pool: ConnectionPool) {
    let app = Router::new()
        .route(
            "/tables/:table_id/order_items",
            get(handler::get_order_items),
        )
        .route(
            "/tables/:table_id/order_items/:order_item_id",
            get(handler::get_order_item),
        )
        .route("/tables/:table_id/orders", post(handler::create_order))
        .route(
            "/tables/:table_id/order_items/:order_item_id",
            delete(handler::delete_order_item),
        )
        .with_state(pool);
    // TODO: Get an address from environment variables
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
