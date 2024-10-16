use crate::db::ConnectionPool;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use tokio_postgres::Row;

/// Response for get_order_item
#[derive(Serialize)]
pub struct OrderItem {
    id: i32,
    table_number: i32,
    menu_item_name: String,
    prep_time_minutes: i32,
}

/// Response for get_order_items
#[derive(Serialize)]
pub struct OrderItems {
    items: Vec<OrderItem>,
}

/// Request body for create_order
#[derive(Deserialize)]
pub struct OrderPostParams {
    menu_item_ids: Vec<i32>,
}

impl From<&Row> for OrderItem {
    fn from(row: &Row) -> Self {
        let id: i32 = row.get("id");
        let table_number: i32 = row.get("table_number");
        let menu_item_name: String = row.get("menu_item_name");
        let prep_time_minutes: i32 = row.get("prep_time_minutes");
        OrderItem {
            id,
            table_number,
            menu_item_name,
            prep_time_minutes,
        }
    }
}

// TODO: Mock db connections for tests

/// Handles GET requests to `/tables/:table_id/order_items` and returns a list of order items for the table
pub fn get_order_items() -> Router<ConnectionPool> {
    async fn handler(
        Path(table_id): Path<i32>,
        State(pool): State<ConnectionPool>,
    ) -> Result<Json<OrderItems>, StatusCode> {
        let conn = pool.get().await.unwrap();
        let rows = conn
            .query(
                r#"
                SELECT toi.*, t.table_number, mi.name menu_item_name FROM table_order_items toi
                INNER JOIN menu_items mi ON toi.menu_item_id = mi.id
                INNER JOIN tables t ON toi.table_id = t.id
                WHERE table_id = $1;
            "#,
                &[&table_id],
            )
            .await
            .map_err(|e| {
                eprintln!("Failed to query table_order_items: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        let items = rows.iter().map(|row| row.into()).collect();
        Ok(Json(OrderItems { items }))
    }

    Router::<ConnectionPool>::new().route("/tables/:table_id/order_items", get(handler))
}

/// Handles GET requests to `/tables/:table_id/order_items/:order_item_id` and returns a single order item
pub fn get_order_item() -> Router<ConnectionPool> {
    async fn handler(
        Path((table_id, order_item_id)): Path<(i32, i32)>,
        State(pool): State<ConnectionPool>,
    ) -> Result<Json<OrderItem>, StatusCode> {
        let conn = pool.get().await.unwrap();
        let order_item: OrderItem = conn
            .query_one(
                r#"
                SELECT toi.*, t.table_number, mi.name menu_item_name FROM table_order_items toi
                INNER JOIN menu_items mi ON toi.menu_item_id = mi.id
                INNER JOIN tables t ON toi.table_id = t.id
                WHERE table_id = $1 AND toi.id = $2;
            "#,
                &[&table_id, &order_item_id],
            )
            .await
            .map_err(|e| {
                eprintln!("Failed to query table_order_items: {}", e);
                StatusCode::NOT_FOUND
            })?
            .borrow()
            .into();
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
        let value_placeholders = params
            .menu_item_ids
            .iter()
            .enumerate()
            .map(|(i, _)| format!("(${}, ${}, ${})", i * 3 + 1, i * 3 + 2, i * 3 + 3))
            .into_iter()
            .collect::<Vec<String>>()
            .join(", ");
        // If there are two menu_item_ids, the query will look like:
        // INSERT INTO table_order_items (table_id, menu_item_id, prep_time_minutes) VALUES ($1, $2, $3), ($4, $5, $6);
        let insert_statement = format!(
            r#"
            INSERT INTO table_order_items (table_id, menu_item_id, prep_time_minutes) VALUES
            {};
        "#,
            value_placeholders
        );
        let values = params
            .menu_item_ids
            .iter()
            .flat_map(|id| vec![&table_id, id, &10]) // TODO: make prep_time_minutes random
            .collect::<Vec<&i32>>();
        let conn = pool.get().await.unwrap();
        conn.execute_raw(&insert_statement, values)
            .await
            .map_err(|e| {
                eprintln!("Failed to insert into table_order_items: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
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
        let conn = pool.get().await.unwrap();
        // Check if the order item exists
        conn.query_one(
            "SELECT 1 FROM table_order_items WHERE table_id = $1 AND id = $2",
            &[&table_id, &order_item_id],
        )
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

        let delete_statement = "DELETE FROM table_order_items WHERE table_id = $1 AND id = $2;";
        conn.execute(delete_statement, &[&table_id, &order_item_id])
            .await
            .map_err(|e| {
                eprintln!("Failed to insert into table_order_items: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        Ok(StatusCode::NO_CONTENT)
    }

    Router::<ConnectionPool>::new().route(
        "/tables/:table_id/order_items/:order_item_id",
        delete(handler),
    )
}
