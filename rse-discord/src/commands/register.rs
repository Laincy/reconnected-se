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

use futures_util::TryFutureExt as FuTryFuturesExt;
use poise::{
    CreateReply, send_reply,
    serenity_prelude::{Color, CreateEmbed, CreateEmbedAuthor, Timestamp},
};
use rse_core::{error::Error as RscError, repo::StockRepository};
use snafu::futures::TryFutureExt;

use crate::{Context, Error, error::RegistrationSnafu};

#[poise::command(slash_command, ephemeral)]
pub async fn register<R: StockRepository>(ctx: Context<'_, R>) -> Result<(), Error> {
    let stock_service = ctx.data();

    let registered_id = tokio::try_join!(
        stock_service
            .register_account(Some(ctx.author().id.into()), None)
            .context(RegistrationSnafu),
        ctx.defer().map_err(Error::from)
    )
    .map(|(id, ())| id);

    match registered_id {
        Ok(id) => {
            let reply = CreateReply::default().embed(
                CreateEmbed::default()
                    .title("Success!")
                    .description("Registered your account")
                    .author(
                        CreateEmbedAuthor::new(id)
                            .icon_url(ctx.author().avatar_url().unwrap_or_default()),
                    )
                    .timestamp(Timestamp::now())
                    .color(Color::DARK_GREEN),
            );

            ctx.send(reply).await?;
            Ok(())
        }
        Err(Error::RegistrationError {
            source: RscError::AccountExists,
        }) => {
            let id = stock_service.disc_to_id(ctx.author().id.into()).await.ok();

            let reply = CreateReply::default()
                .embed(
                    CreateEmbed::default()
                        .title("Already exists")
                        .description("You already have an account")
                        .author(
                            CreateEmbedAuthor::new(id.unwrap_or_default())
                                .icon_url(ctx.author().avatar_url().unwrap_or_default()),
                        )
                        .timestamp(Timestamp::now())
                        .color(Color::BLITZ_BLUE),
                )
                .ephemeral(true);

            send_reply(ctx, reply).await?;

            Ok(())
        }
        // Pass on to global error handling
        Err(error) => Err(error),
    }
}
