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

use poise::{
    CreateReply, send_reply,
    serenity_prelude::{
        Color, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter,
        CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage, User,
    },
};
use rse_core::{
    model::{Pager, ticker::Ticker},
    repo::StockRepository,
};
use std::{fmt::Write, ops::Rem};

use crate::{Context, Error};

#[poise::command(slash_command, ephemeral)]
#[allow(clippy::too_many_lines)]
pub async fn portfolio<R: StockRepository>(
    ctx: Context<'_, R>,
    #[description = "Selected user"] user: Option<User>,
) -> Result<(), Error> {
    const PAGE_SIZE: i64 = 16;
    let stock_service = ctx.data();
    let user = user.unwrap_or(ctx.author().clone());
    let user_id = ctx.data().disc_to_id(user.id.into()).await?;
    let ctx_id = ctx.id();

    let prev_button_id = format!("{ctx_id}prev");
    let next_button_id = format!("{ctx_id}next");

    let mut page = Pager::new(0, PAGE_SIZE);

    let ((holdings, num_entries), info) = tokio::try_join!(
        stock_service.get_holdings(&user_id, &page),
        stock_service.get_account_info(&user_id)
    )?;

    let mut current_page: i64 = 0;
    let total_pages = num_entries / PAGE_SIZE + num_entries.rem(PAGE_SIZE).clamp(0, 1);

    // Fucking serenity will make me clone this every time because it doesn't like references :(
    let reply_embed = CreateEmbed::new()
        .color(Color::BLITZ_BLUE)
        .thumbnail(user.avatar_url().unwrap_or_default())
        .author(CreateEmbedAuthor::new(user.display_name()))
        .field("Balance", info.balance.to_string(), true)
        .field(
            "Created",
            info.created_at.format("%Y-%m-%d %H:%M").to_string(),
            true,
        )
        .description(into_page(&holdings));

    match total_pages {
        0 => {
            send_reply(
                ctx,
                CreateReply::default().embed(
                    reply_embed
                        .field(
                            "Holdings",
                            "This user has not purchased any stocks yet",
                            false,
                        )
                        .footer(CreateEmbedFooter::new(user_id)),
                ),
            )
            .await?;

            return Ok(());
        }
        1 => {
            send_reply(
                ctx,
                CreateReply::default().embed(
                    reply_embed
                        .field("Holdings", into_page(&holdings), false)
                        .footer(CreateEmbedFooter::new(user_id)),
                ),
            )
            .await?;

            return Ok(());
        }
        _ => {
            let components = CreateActionRow::Buttons(vec![
                CreateButton::new(&prev_button_id).emoji('◀'),
                CreateButton::new(&next_button_id).emoji('▶'),
            ]);

            send_reply(
                ctx,
                CreateReply::default()
                    .embed(
                        reply_embed
                            .clone()
                            .field("Holdings", into_page(&holdings), false)
                            .footer(CreateEmbedFooter::new(format!(
                                "Page: {}/{} - {user_id}",
                                current_page + 1,
                                total_pages + 1
                            ))),
                    )
                    .components(vec![components]),
            )
            .await?;
        }
    }

    while let Some(press) =
        poise::serenity_prelude::collector::ComponentInteractionCollector::new(ctx)
            .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
            .timeout(std::time::Duration::from_secs(1800))
            .await
    {
        tracing::info!("Pressed! {}", press.data.custom_id);
        if press.data.custom_id == prev_button_id {
            current_page = current_page.checked_sub(1).unwrap_or(total_pages);
        } else if press.data.custom_id == next_button_id {
            current_page += 1;
            current_page %= total_pages + 1;
        } else {
            // Unrelated interaction
            continue;
        }

        page.set_offset(current_page * PAGE_SIZE);

        let (holdings, new_entries) = stock_service.get_holdings(&user_id, &page).await?;

        if new_entries != num_entries {
            press
                .create_followup(
                    ctx.serenity_context(),
                    CreateInteractionResponseFollowup::new()
                        .content("User's holdings have changed, please call again"),
                )
                .await?;
            break;
        }
        press
            .create_response(
                ctx.serenity_context(),
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new().embed(
                        reply_embed
                            .clone()
                            .footer(CreateEmbedFooter::new(format!(
                                "Page: {}/{} - {user_id}",
                                current_page + 1,
                                total_pages + 1
                            )))
                            .field("Holdings", into_page(&holdings), false),
                    ),
                ),
            )
            .await?;
    }
    Ok(())
}

fn into_page(v: &[(Ticker, u32)]) -> String {
    let mut buff = String::new();

    for (ticker, shares) in v {
        writeln!(buff, "${}: {}", ticker.as_str(), shares).expect("Never fails");
    }

    buff
}
