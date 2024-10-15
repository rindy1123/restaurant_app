#![warn(clippy::all)]

use std::{borrow::Borrow, ops::DerefMut};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use serde::{Deserialize, Serialize};
use tokio_postgres::{NoTls, Row};

#[derive(Serialize)]
struct OrderItem {
    id: i32,
    table_number: i32,
    menu_item_name: String,
    prep_time_minutes: i32,
}

#[derive(Serialize)]
struct OrderItems {
    items: Vec<OrderItem>,
}

#[derive(Deserialize)]
struct OrderPostParams {
    menu_item_ids: Vec<i32>,
}

type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

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

async fn get_order_items(
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

async fn get_order_item(
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

async fn create_order(
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

async fn delete_order_item(
    Path((table_id, order_item_id)): Path<(i32, i32)>,
    State(pool): State<ConnectionPool>,
) -> Result<StatusCode, StatusCode> {
    let conn = pool.get().await.unwrap();
    conn.query(
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

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

#[tokio::main]
async fn main() {
    // Database setup
    // TODO: Get db info from environment variables
    let manager = PostgresConnectionManager::new_from_stringlike(
        "host=postgres user=postgres password=postgres dbname=restaurant_app",
        NoTls,
    )
    .unwrap();
    let pool = Pool::builder().build(manager).await.unwrap();
    let mut conn = pool.get().await.unwrap();
    let client = conn.deref_mut();
    embedded::migrations::runner()
        .run_async(client)
        .await
        .unwrap();

    // Web server setup
    let pool = pool.clone();
    let app = Router::new()
        .route("/tables/:table_id/order_items", get(get_order_items))
        .route(
            "/tables/:table_id/order_items/:order_item_id",
            get(get_order_item),
        )
        .route("/tables/:table_id/orders", post(create_order))
        .route(
            "/tables/:table_id/order_items/:order_item_id",
            delete(delete_order_item),
        )
        .with_state(pool);
    // TODO: Get an address from environment variables
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
