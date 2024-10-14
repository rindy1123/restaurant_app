#![warn(clippy::all)]

use axum::{
    extract::Path,
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio_postgres::NoTls;

#[derive(Serialize)]
struct OrderItem {
    id: u64,
    name: String,
    price: f64,
}

#[derive(Deserialize)]
struct OrderPostParam {
    menu_item_ids: Vec<u64>,
}

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

async fn create_order(Path(table_id): Path<u64>, Json(param): Json<OrderPostParam>) -> StatusCode {
    println!("{table_id}");
    for id in param.menu_item_ids {
        println!("{id}");
    }
    StatusCode::CREATED
}

async fn delete_order_item(Path((table_id, order_item_id)): Path<(u64, u64)>) -> StatusCode {
    println!("{table_id}/{order_item_id}");
    StatusCode::NO_CONTENT
}

#[tokio::main]
async fn main() -> Result<(), tokio_postgres::Error> {
    let (_client, connection) = tokio_postgres::connect(
        "host=postgres user=postgres password=postgres dbname=restaurant_app",
        NoTls,
    )
    .await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

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
        );
    // TODO: Get an address from environment variables
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
