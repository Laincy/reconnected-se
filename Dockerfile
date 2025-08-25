FROM rust:1.89.0-slim-bookworm AS base
RUN cargo install cargo-chef --locked


FROM base AS planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM base AS builder
WORKDIR /app
ARG SQLX_OFFLINE=true
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin rse-server


FROM debian:bookworm-slim as runtime
WORKDIR /rse
copy migrations .
COPY --from=builder /app/target/release/rse-server /usr/local/bin
CMD ["rse-server"]
