# Reconnected Stock Exchange (RSE)

RSE is a project trying to create a stock exchange for [Reconnected CC](reconnected.cc). It is licensed under AGPL-v3 and written in Rust.

Running RSE is as simple as git cloning this repo, and running

```sh
docker compose up
```

## Project Structure

This project (vaguely) tries to follow a hexagonal architecture.

- `src`: Server, imports other crates and starts them up as one cohesive unit
- `rse-core`: The core implementation, creating all services that other crates build upon or implement. Also includes the implementation for our database port
- `rse-discord`: Our discord implementation, such as our bot and `webhook` client

## Development

Creating a development environment for RSE takes a couple of steps

1. **Install and configure the following**

    - [Rust](https://www.rust-lang.org/learn/get-started)
    - [sqlx-cli](https://crates.io/crates/sqlx-cli)
    - [Docker](https://www.docker.com/) or [Podman](https://podman.io/)
    - (Optional) [cargo-deny](https://crates.io/crates/cargo-deny)

2. **Create a `.env` file**

    Use the same format as `.env.example`. You can keep the `DATABASE_URL` as it is, or change it and the environment variables specified in `compose.yaml`.

3. **Start the database**

    Run the following command to start the database and run migrations on it

    ```sh
    docker compose up -d database && cargo sqlx migrate run
    ```

You should now have a working development environment. Before you build the docker image or commit to git, make sure to run

### SQLX queries

```sh
  cargo sqlx prepare --workspace
```

Run this before you commit to git or build the Docker image. It must be done while you have a connection to an existing database. Without it, sqlx's compile time checks will fail and the container will not build.
