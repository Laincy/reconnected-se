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

//! Errors for the `RSE` Discord integration

use poise::{
    BoxFuture, CreateReply, FrameworkError,
    serenity_prelude::{Color, CreateEmbed, Timestamp},
};
use snafu::Snafu;

/// Poise result type
use rse_core::{Service, error::Error as RscErr, repo::StockRepository};
use tracing::Level;

/// Errors emitted by the discord integration. Need to sanitize this so it can be exposed back to
/// Discord users
#[derive(Debug, Snafu)]
#[allow(missing_docs)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    /// Errors emitted by internal errors
    #[snafu(transparent)]
    ServiceError { source: RscErr },

    /// Errors emitted by poise itself
    #[snafu(transparent)]
    PoiseError {
        source: poise::serenity_prelude::Error,
    },

    #[snafu(display("Could not register your account!"))]
    RegistrationError { source: RscErr },
}

pub fn on_error<R: StockRepository>(
    error: FrameworkError<'_, Service<R>, Error>,
) -> BoxFuture<'_, ()> {
    Box::pin(async move {
        let span = tracing::span!(Level::TRACE, "on_error", error = error.to_string());
        let _guard = span.enter();
        match error {
            FrameworkError::Command { error, ctx, .. } => {
                let mut reply_embed = CreateEmbed::new()
                    .title("Error!")
                    .color(Color::RED)
                    .timestamp(Timestamp::now());
                match error {
                    Error::RegistrationError { source } => {
                        reply_embed = reply_embed
                            .description(
                                "Could not register your account, please try again later! If the issue persists, contact support.",
                            );
                        tracing::error!("couldn't register account: {source:?}");
                    }
                    Error::ServiceError {
                        source: RscErr::UserNotFound,
                    } => {
                        reply_embed = reply_embed.description("This user does not have an account");
                    }
                    _ => {
                        reply_embed = reply_embed.description(
                            "Experienced an unexpected internal error, please try again later! If the issue persists, contact support.",
                        );
                    }
                }

                if let Err(res_err) = ctx
                    .send(CreateReply::default().embed(reply_embed).ephemeral(true))
                    .await
                {
                    tracing::warn!("Could not error gracefully: {res_err}");
                } else {
                    tracing::trace!("Responded to error gracefully");
                }
            }
            FrameworkError::CommandPanic { payload, ctx, .. } => {
                tracing::error!({ payload = payload }, "Panicked inside command");
                let reply = CreateReply::default().embed(CreateEmbed::new()
                    .title("Error!")
                    .color(Color::RED)
                    .timestamp(Timestamp::now())
                    .description(
                            "Experienced an internal error, please try again later! If the issue persists, contact support.",
                    )).ephemeral(true);

                if let Err(res_err) = ctx.send(reply).await {
                    tracing::warn!("Could not error gracefully: {res_err}");
                } else {
                    tracing::trace!("Responded to error gracefully");
                }
            }
            _ => tracing::warn!("Experienced a Discord Error"),
        }
    })
}
