use crate::{
    db::ConnectionPool,
    server::table_order_items::{self, OrderItems, TableOrderItemError},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;

use super::table_order_items::OrderItem;

/// Request body for create_order
#[derive(Deserialize)]
pub struct OrderPostParams {
    menu_item_ids: Vec<i32>,
}

/// Handles GET requests to `/tables/:table_id/order_items` and returns a list of order items for the table
pub fn get_order_items() -> Router<ConnectionPool> {
    async fn handler(
        Path(table_id): Path<i32>,
        State(pool): State<ConnectionPool>,
    ) -> Result<Json<OrderItems>, StatusCode> {
        println!("GET: /tables/{}/order_items", table_id);
        let order_items = table_order_items::get_order_items(&pool, table_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Json(order_items))
    }

    Router::<ConnectionPool>::new().route("/tables/:table_id/order_items", get(handler))
}

/// Handles GET requests to `/tables/:table_id/order_items/:order_item_id` and returns a single order item
pub fn get_order_item() -> Router<ConnectionPool> {
    async fn handler(
        Path((table_id, order_item_id)): Path<(i32, i32)>,
        State(pool): State<ConnectionPool>,
    ) -> Result<Json<OrderItem>, StatusCode> {
        println!("GET: /tables/{}/order_items/{}", table_id, order_item_id);
        let order_item = table_order_items::get_order_item(&pool, table_id, order_item_id)
            .await
            .map_err(|e| match e {
                TableOrderItemError::NotFoundError => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })?;
        Ok(Json(order_item))
    }

    Router::<ConnectionPool>::new()
        .route("/tables/:table_id/order_items/:order_item_id", get(handler))
}

/// Handles POST requests to `/tables/:table_id/orders` and creates a new order for the table
pub fn create_order() -> Router<ConnectionPool> {
    async fn handler(
        Path(table_id): Path<i32>,
        State(pool): State<ConnectionPool>,
        Json(params): Json<OrderPostParams>,
    ) -> Result<StatusCode, StatusCode> {
        println!("POST: /tables/{}/orders", table_id);
        table_order_items::insert_table_order_items(&pool, params.menu_item_ids, table_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(StatusCode::CREATED)
    }

    Router::<ConnectionPool>::new().route("/tables/:table_id/orders", post(handler))
}

/// Handles DELETE requests to `/tables/:table_id/order_items/:order_item_id` and deletes an order item
pub fn delete_order_item() -> Router<ConnectionPool> {
    async fn handler(
        Path((table_id, order_item_id)): Path<(i32, i32)>,
        State(pool): State<ConnectionPool>,
    ) -> Result<StatusCode, StatusCode> {
        println!("DELETE: /tables/{}/order_items/{}", table_id, order_item_id);
        let conn = pool.get().await.unwrap();
        // Check if the order item exists
        conn.query_one(
            "SELECT 1 FROM table_order_items WHERE table_id = $1 AND id = $2",
            &[&table_id, &order_item_id],
        )
        .await
        .map_err(|_| {
            println!("table_order_items (id: {order_item_id}) not found");
            StatusCode::NOT_FOUND
        })?;

        let delete_statement = "DELETE FROM table_order_items WHERE table_id = $1 AND id = $2;";
        conn.execute(delete_statement, &[&table_id, &order_item_id])
            .await
            .map_err(|e| {
                eprintln!("Failed to delete table_order_items: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        Ok(StatusCode::NO_CONTENT)
    }

    Router::<ConnectionPool>::new().route(
        "/tables/:table_id/order_items/:order_item_id",
        delete(handler),
    )
}
