/*
    This file is part of the Reconnected Stock Exchange (RSE)
    Copyright (C) 2025 The RSE Team

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

#![allow(missing_docs)]

use rse_core::{Service, repo::PgPort};
use tokio::{
    select,
    signal::unix::{SignalKind, signal},
};
use tokio_util::sync::CancellationToken;
use tracing::info;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    let cancel_token = CancellationToken::new();

    dotenvy::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("'DATABASE_URL' not set");

    let pool = sqlx::PgPool::connect(&db_url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let service = Service::new(PgPort::new(pool));

    let disc_handle = rse_discord::start(service, cancel_token.clone()).await;

    // Graceful shutdown stuff
    let mut sigterm = signal(SignalKind::terminate()).expect("Couldn't create handle for SIGTERM");
    let mut sigint = signal(SignalKind::interrupt()).expect("Couldn't create handle for SIGINT");
    select! {
        _ = sigterm.recv() => info!("Received SIGTERM"),
        _ = sigint.recv()=> info!("Received SIGINT"),
    };

    cancel_token.cancel();

    disc_handle.await?;

    info!("Gracefully shutdown");

    Ok(())
}
