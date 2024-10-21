use std::{env, ops::DerefMut};

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use refinery::embed_migrations;
use tokio_postgres::NoTls;

embed_migrations!("./migrations");

pub type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

pub async fn get_connection_pool() -> ConnectionPool {
    let host = env::var("DB_HOST").unwrap();
    let user = env::var("DB_USER").unwrap();
    let password = env::var("DB_PASSWORD").unwrap();
    let dbname = env::var("DB_NAME").unwrap();
    let manager = PostgresConnectionManager::new_from_stringlike(
        format!("host={host} user={user} password={password} dbname={dbname}"),
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
