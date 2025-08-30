use std::ops::Rem;

use crate::{Context, Error};
use chrono::{DateTime, Utc};
use poise::{
    CreateReply, send_reply,
    serenity_prelude::{
        Color, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter,
        CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage,
    },
};
use rse_core::{
    model::{Pager, ticker::Ticker},
    repo::StockRepository,
};
use rust_decimal::Decimal;

#[poise::command(slash_command, ephemeral)]
pub async fn stocks<R: StockRepository>(ctx: Context<'_, R>) -> Result<(), Error> {
    const PAGE_SIZE: i64 = 16;
    let ctx_id = ctx.id();
    let stock_service = ctx.data();

    let mut page = Pager::new(0, PAGE_SIZE);

    let res = stock_service.list_stocks(&page).await;

    if let Err(e) = res
        && e == rse_core::error::Error::NoStocksExist
    {
        send_reply(
            ctx,
            CreateReply::default().embed(
                CreateEmbed::new()
                    .description("No stock data to display")
                    .color(Color::BLURPLE),
            ),
        )
        .await?;

        return Ok(());
    }

    let (stocks, num_entries) = res?;

    let total_pages = num_entries / PAGE_SIZE + num_entries.rem(PAGE_SIZE).clamp(0, 1);

    if total_pages == 1 {
        send_reply(ctx, CreateReply::default().embed(into_embed(&stocks))).await?;
        return Ok(());
    }

    let mut current_page: i64 = 0;
    let prev_button_id = format!("{ctx_id}prev");
    let next_button_id = format!("{ctx_id}next");

    let reply = {
        let components = CreateActionRow::Buttons(vec![
            CreateButton::new(&prev_button_id).emoji('◀'),
            CreateButton::new(&next_button_id).emoji('▶'),
        ]);

        CreateReply::default()
            .embed(
                into_embed(&stocks)
                    .footer(CreateEmbedFooter::new(format!("Page: 1/{total_pages}"))),
            )
            .components(vec![components])
    };

    send_reply(ctx, reply).await?;

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

        let (stocks, new_entries) = stock_service.list_stocks(&page).await?;

        if new_entries != num_entries {
            press
                .create_followup(
                    ctx.serenity_context(),
                    CreateInteractionResponseFollowup::new()
                        .content("A new stock has been created, please call this command again"),
                )
                .await?;
            break;
        }

        press
            .create_response(
                ctx.serenity_context(),
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new().embed(into_embed(&stocks).footer(
                        CreateEmbedFooter::new(format!("Page: {current_page}/{total_pages}")),
                    )),
                ),
            )
            .await?;
    }

    Ok(())
}

fn into_embed(v: &[(Ticker, u32, Decimal, DateTime<Utc>)]) -> CreateEmbed {
    let fields = v.iter().map(|(ticker, shares, value, time)| {
        (
            ticker.as_str(),
            format!("Shares: {shares}\nPrice: {value}\nLast Sold: {time}"),
            true,
        )
    });

    CreateEmbed::new().color(Color::BLURPLE).fields(fields)
}
