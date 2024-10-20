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

First, make sure that the containers are running:

```bash
docker compose ps
```

You'll see the following output something like this:
```bash
NAME                                   IMAGE                           COMMAND                  SERVICE    CREATED          STATUS          PORTS
restaurant_app_assignment-api-1        restaurant_app_assignment-api   "cargo watch -x run"     api        34 minutes ago   Up 34 minutes   58000/tcp, 0.0.0.0:58000->8000/tcp, [::]:58000->8000/tcp
restaurant_app_assignment-postgres-1   postgres:16-alpine              "docker-entrypoint.s…"   postgres   34 minutes ago   Up 34 minutes   0.0.0.0:55432->5432/tcp, [::]:55432->5432/tcp
```

Then, run the following command to run the tests:

```bash
make test
```

## Caveats

I left a few TODO comments which I consider trivial or too much to implement for this assignment. These include mocking the database for testing and fetching some data from environment variables which is currently hard coded. 
