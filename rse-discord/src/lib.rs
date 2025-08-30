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

//! Discord adapters for the `RSE` program

use poise::serenity_prelude::{self as serenity, GuildId, OnlineStatus};
use rse_core::{Service, repo::StockRepository};

pub use error::Error;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::info;

mod commands;
mod error;

/// Context of the discord runner
pub type Context<'a, R> = poise::Context<'a, Service<R>, Error>;

/// Start the discord bot task
pub async fn start<R: StockRepository>(
    service: Service<R>,
    c_token: CancellationToken,
) -> JoinHandle<()> {
    let token =
        std::env::var("DISCORD_TOKEN").expect("'Discord_TOKEN' environment variable is not set");

    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                about(),
                commands::register(),
                commands::portfolio(),
                commands::stocks(),
            ],
            on_error: error::on_error,
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    GuildId::new(1_408_958_403_438_444_746),
                )
                .await?;
                Ok(service)
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .expect("Couldn't create client");

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::select! {
            () = c_token.cancelled() => {
                {
                    let runners = shard_manager.runners.lock().await;
                    for (_id, runner) in runners.iter() {
                        runner.runner_tx.set_status(OnlineStatus::Offline);
                    }
                }

                info!("Shutting down Discord bot");
                shard_manager.shutdown_all().await;
            },
            res = client.start()  => {
                if let Err(why) = res {
                    tracing::error!("{why}");
                }
            }
        }
    })
}

#[poise::command(slash_command, prefix_command, guild_only)]
async fn about<R: StockRepository>(ctx: Context<'_, R>) -> Result<(), Error> {
    let reply = poise::CreateReply::default().ephemeral(true).content(
        "A discord bot that manages the Reconnected Stock Exchange.\nLicensed under AGPL-3.0",
    );
    ctx.send(reply).await?;

    Ok(())
}
