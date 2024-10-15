#![warn(clippy::all)]

use std::ops::DerefMut;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use serde::{Deserialize, Serialize};
use tokio_postgres::NoTls;

#[derive(Serialize)]
struct OrderItem {
    id: u64,
    name: String,
    price: f64,
}

#[derive(Deserialize)]
struct OrderPostParams {
    menu_item_ids: Vec<i32>,
}

type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

async fn get_order_items(Path(table_id): Path<u64>) -> Json<OrderItem> {
    let item = OrderItem {
        id: table_id,
        name: "Table".to_string(),
        price: 100.0,
    };
    Json(item)
}

async fn get_order_item(Path((table_id, order_item_id)): Path<(u64, u64)>) -> Json<OrderItem> {
    println!("{}", order_item_id);
    let item = OrderItem {
        id: table_id,
        name: "Table".to_string(),
        price: 100.0,
    };
    Json(item)
}

async fn create_order(
    Path(table_id): Path<i32>,
    State(pool): State<ConnectionPool>,
    Json(params): Json<OrderPostParams>,
) -> StatusCode {
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
        .flat_map(|id| vec![&table_id, id, &10])
        .collect::<Vec<&i32>>();
    let conn = pool.get().await.unwrap();
    if let Err(e) = conn.execute_raw(&insert_statement, values).await {
        eprintln!("Failed to insert into orders: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::CREATED
}

async fn delete_order_item(Path((table_id, order_item_id)): Path<(u64, u64)>) -> StatusCode {
    println!("{table_id}/{order_item_id}");
    StatusCode::NO_CONTENT
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
