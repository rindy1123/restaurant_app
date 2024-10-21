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

#[cfg(test)]
pub mod test_utils {
    use testcontainers::{runners::AsyncRunner, ContainerAsync};
    use testcontainers_modules::postgres::Postgres;

    use super::*;

    // NOTE: If you create a container inside this function, `drop` trait will kill the container
    // and you won't be able to connect to the database.
    pub async fn set_up_test_db(container: &ContainerAsync<Postgres>) -> ConnectionPool {
        let port = container.get_host_port_ipv4(5432).await.unwrap();

        let manager = PostgresConnectionManager::new_from_stringlike(
            format!("host=localhost port={port} user=postgres password=postgres dbname=postgres"),
            NoTls,
        )
        .unwrap();
        let pool = Pool::builder().build(manager).await.unwrap();
        run_migrations(&pool).await;
        pool
    }

    // NOTE: Some tests which requires a database connection use a container to build an independent database instance for testing.
    pub async fn create_db_container() -> ContainerAsync<Postgres> {
        Postgres::default().start().await.unwrap()
    }
}
