use std::borrow::Borrow;

use rand::Rng;
use serde::Serialize;
use tokio_postgres::Row;

use crate::db::ConnectionPool;

#[derive(Serialize)]
pub struct OrderItem {
    id: i32,
    table_number: i32,
    menu_item_name: String,
    prep_time_minutes: i32,
}

#[derive(Serialize)]
pub struct OrderItems {
    items: Vec<OrderItem>,
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

#[derive(Debug, PartialEq)]
pub enum TableOrderItemError {
    PoolError,
    QueryError,
    NotFoundError,
}

pub async fn get_order_items(
    pool: &ConnectionPool,
    table_id: i32,
) -> Result<OrderItems, TableOrderItemError> {
    let conn = pool.get().await.map_err(|e| {
        eprintln!("Failed to get connection from pool: {}", e);
        TableOrderItemError::PoolError
    })?;
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
            TableOrderItemError::QueryError
        })?;
    let items = rows.iter().map(|row| row.into()).collect();
    Ok(OrderItems { items })
}

pub async fn get_order_item(
    pool: &ConnectionPool,
    table_id: i32,
    order_item_id: i32,
) -> Result<OrderItem, TableOrderItemError> {
    let conn = pool.get().await.map_err(|e| {
        eprintln!("Failed to get connection from pool: {}", e);
        TableOrderItemError::PoolError
    })?;
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
        .map_err(|_| {
            println!("table_order_items (id: {order_item_id}) not found");
            TableOrderItemError::NotFoundError
        })?
        .borrow()
        .into();
    Ok(order_item)
}

pub async fn insert_table_order_items(
    pool: &ConnectionPool,
    menu_item_ids: Vec<i32>,
    table_id: i32,
) -> Result<(), TableOrderItemError> {
    let conn = pool.get().await.map_err(|e| {
        eprintln!("Failed to get connection from pool: {}", e);
        TableOrderItemError::PoolError
    })?;
    let value_placeholders = generate_value_placeholders_for_insert_statement(menu_item_ids.len());
    // If there are two menu_item_ids, the query will look like:
    // INSERT INTO table_order_items (table_id, menu_item_id, prep_time_minutes) VALUES ($1, $2, $3), ($4, $5, $6);
    let insert_statement = format!(
        r#"
            INSERT INTO table_order_items (table_id, menu_item_id, prep_time_minutes) VALUES
            {};
        "#,
        value_placeholders
    );
    let prep_time_minutes = get_random_prep_time_minutes();
    let values = menu_item_ids
        .iter()
        .flat_map(|id| vec![&table_id, id, &prep_time_minutes])
        .collect::<Vec<&i32>>();
    conn.execute_raw(&insert_statement, values)
        .await
        .map_err(|e| {
            eprintln!("Failed to insert table_order_items: {}", e);
            TableOrderItemError::QueryError
        })?;
    Ok(())
}

pub async fn delete_order_item(
    pool: &ConnectionPool,
    table_id: i32,
    order_item_id: i32,
) -> Result<(), TableOrderItemError> {
    let conn = pool.get().await.map_err(|e| {
        eprintln!("Failed to get connection from pool: {}", e);
        TableOrderItemError::PoolError
    })?;
    // Check if the order item exists
    conn.query_one(
        "SELECT 1 FROM table_order_items WHERE table_id = $1 AND id = $2",
        &[&table_id, &order_item_id],
    )
    .await
    .map_err(|_| {
        println!("table_order_items (id: {order_item_id}) not found");
        TableOrderItemError::NotFoundError
    })?;

    let delete_statement = "DELETE FROM table_order_items WHERE table_id = $1 AND id = $2;";
    conn.execute(delete_statement, &[&table_id, &order_item_id])
        .await
        .map_err(|e| {
            eprintln!("Failed to delete table_order_items: {}", e);
            TableOrderItemError::QueryError
        })?;
    Ok(())
}

/// Returns a random prep time between 5 and 15 minutes
fn get_random_prep_time_minutes() -> i32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(5..=15)
}

fn generate_value_placeholders_for_insert_statement(num_of_items: usize) -> String {
    (0..num_of_items)
        .map(|i| format!("(${}, ${}, ${})", i * 3 + 1, i * 3 + 2, i * 3 + 3))
        .into_iter()
        .collect::<Vec<String>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use crate::db::test_utils::{create_db_container, set_up_test_db};

    use super::*;

    #[tokio::test]
    async fn test_insert_table_order_items() {
        let container = create_db_container().await;
        let pool = set_up_test_db(&container).await;
        let menu_item_ids = vec![1, 2, 3];
        let table_id = 1;
        insert_table_order_items(&pool, menu_item_ids, table_id)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_get_order_item() {
        let container = create_db_container().await;
        let pool = set_up_test_db(&container).await;
        let menu_item_ids = vec![1];
        let table_id = 1;
        insert_table_order_items(&pool, menu_item_ids, table_id)
            .await
            .unwrap();
        let order_item = get_order_item(&pool, table_id, 1).await.unwrap();
        assert_eq!(order_item.id, 1);
        assert_eq!(order_item.table_number, 1);
        assert_eq!(order_item.menu_item_name, "Big Mac");
        assert!(order_item.prep_time_minutes >= 5 && order_item.prep_time_minutes <= 15);
    }

    #[tokio::test]
    async fn test_get_order_items() {
        let container = create_db_container().await;
        let pool = set_up_test_db(&container).await;
        let menu_item_ids = vec![1, 2, 3];
        let table_id = 1;
        insert_table_order_items(&pool, menu_item_ids, table_id)
            .await
            .unwrap();
        let OrderItems { items } = get_order_items(&pool, table_id).await.unwrap();
        assert_eq!(items.len(), 3);
    }

    #[tokio::test]
    async fn test_get_order_item_not_found() {
        let container = create_db_container().await;
        let pool = set_up_test_db(&container).await;
        let table_id = 1;
        let result = get_order_item(&pool, table_id, 1).await.err().unwrap();
        let expected = TableOrderItemError::NotFoundError;
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_delete_order_item() {
        let container = create_db_container().await;
        let pool = set_up_test_db(&container).await;
        let menu_item_ids = vec![1];
        let table_id = 1;
        insert_table_order_items(&pool, menu_item_ids, table_id)
            .await
            .unwrap();
        delete_order_item(&pool, table_id, 1).await.unwrap();
        let OrderItems { items } = get_order_items(&pool, table_id).await.unwrap();
        assert_eq!(items.len(), 0);
    }

    #[tokio::test]
    async fn test_delete_order_item_not_found() {
        let container = create_db_container().await;
        let pool = set_up_test_db(&container).await;
        let table_id = 1;
        let result = delete_order_item(&pool, table_id, 1).await.err().unwrap();
        let expected = TableOrderItemError::NotFoundError;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_random_prep_time_minutes() {
        let prep_time = get_random_prep_time_minutes();
        assert!(prep_time >= 5 && prep_time <= 15);
    }

    #[test]
    fn test_generate_value_placeholders_for_insert_statement() {
        let value_placeholders = generate_value_placeholders_for_insert_statement(3);
        assert_eq!(
            value_placeholders,
            "($1, $2, $3), ($4, $5, $6), ($7, $8, $9)"
        );
    }
}
