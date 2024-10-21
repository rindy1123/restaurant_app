# Restaurant App Assignment

## Description

This is a backend application designed to streamline restaurant operations by allowing staff to efficiently manage table orders. The app provides functionality to store, retrieve, and remove menu items for specific tables, along with tracking the preparation time for each item.

For detailed information on the endpoints and ERD, please refer to [the files under ./documents](./documents).

## Dependencies


- [Docker](https://docs.docker.com/get-docker/): Containerizes both the application and the database for ease of deployment and consistent development environments.
- [axum](https://github.com/tokio-rs/axum): A Rust-based web framework for building efficient and scalable HTTP services.
- [tokio-postgres](https://docs.rs/tokio-postgres/latest/tokio_postgres/): A non-blocking PostgreSQL client for handling database operations asynchronously.
- [refinery](https://github.com/rust-db/refinery): A database migration tool that helps manage schema changes.

## Development environment

We containerize the development environment. We have 2 containers so far:
- api
    - The main container that runs the application
- db
    - The database container

### Setup

We use `docker compose` to manage the development environment. To start the app, run the following command:

```bash
docker compose up -d
```

The app will be running on `http://localhost:58000`.

### Testing

First, make sure that `cargo` is installed by running:

```bash
cargo -V
```

Then, run the following command to run the tests:
```bash
cargo test
```
