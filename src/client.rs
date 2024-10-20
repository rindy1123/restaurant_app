use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize)]
#[allow(dead_code)]
struct OrderItem {
    id: i32,
    table_number: i32,
    menu_item_name: String,
    prep_time_minutes: i32,
}

#[derive(Deserialize)]
struct OrderItems {
    items: Vec<OrderItem>,
}

pub async fn mock_requests() {
    let client = reqwest::Client::new();
    let mut json_body = HashMap::new();
    json_body.insert("menu_item_ids", vec![1, 2, 3]);
    // TODO: table_id should be fetched from some API like GET /tables
    // Order some items
    client
        .post("http://localhost:8000/tables/21/orders")
        .json(&json_body)
        .send()
        .await
        .unwrap();
    // Retrieve the order items for the table
    let OrderItems { items } = client
        .get("http://localhost:8000/tables/21/order_items")
        .send()
        .await
        .unwrap()
        .json::<OrderItems>()
        .await
        .unwrap();
    // Retrieve the first order item
    let order_item_id = items[0].id;
    client
        .get(format!(
            "http://localhost:8000/tables/21/order_items/{}",
            order_item_id
        ))
        .send()
        .await
        .unwrap();
    // Delete the item
    client
        .delete(format!(
            "http://localhost:8000/tables/21/order_items/{}",
            order_item_id
        ))
        .send()
        .await
        .unwrap();
}
