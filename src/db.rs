use std::ops::DerefMut;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use refinery::embed_migrations;
use tokio_postgres::NoTls;

embed_migrations!("./migrations");

pub type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

pub async fn get_connection_pool() -> ConnectionPool {
    // TODO: Get db info from environment variables
    let manager = PostgresConnectionManager::new_from_stringlike(
        "host=postgres user=postgres password=postgres dbname=restaurant_app",
        NoTls,
    )
    .unwrap();
    Pool::builder().build(manager).await.unwrap()
}

/// Ref: [Refinery: deadpool](https://github.com/rust-db/refinery?tab=readme-ov-file#example-deadpool)
pub async fn run_migrations(pool: &ConnectionPool) {
    let mut conn = pool.get().await.unwrap();
    let client = conn.deref_mut();
    migrations::runner().run_async(client).await.unwrap();
}
