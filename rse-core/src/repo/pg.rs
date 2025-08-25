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

use std::num::NonZeroU64;

use chrono::{DateTime, Utc};
use futures_util::{FutureExt, TryFutureExt};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::model::ticker::Ticker;
use crate::model::{Pager, UserInfo};
use crate::repo::Error;

/// A port for a `Postgres` back end
#[derive(Debug, Clone)]
pub struct PgPort {
    pool: sqlx::PgPool,
}

impl PgPort {
    /// Creates a new instance of [`PgPort`]
    #[must_use]
    pub const fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}
impl super::StockRepository for PgPort {
    fn user_exists(&self, id: &uuid::Uuid) -> impl Future<Output = super::Result<bool>> + Send {
        sqlx::query_scalar!("SELECT EXISTS (SELECT 1 FROM users WHERE user_id = $1)", id)
            .fetch_one(&self.pool)
            .map(|res| match res {
                Ok(b) => Ok(b.unwrap_or_default()),
                Err(_) => Err(Error::Unspecified),
            })
    }

    fn stock_exists(&self, stock: &Ticker) -> impl Future<Output = super::Result<bool>> + Send {
        sqlx::query_scalar!(
            "SELECT EXISTS (SELECT 1 FROM stocks WHERE ticker = $1)",
            stock.as_str()
        )
        .fetch_one(&self.pool)
        .map(|res| match res {
            Ok(b) => Ok(b.unwrap_or_default()),
            Err(_) => Err(Error::Unspecified),
        })
    }

    fn discord_to_id(
        &self,
        id: i64,
    ) -> impl Future<Output = super::Result<Option<uuid::Uuid>>> + Send {
        sqlx::query_scalar!("SELECT user_id FROM users where disc_id = $1", id)
            .fetch_optional(&self.pool)
            .map_err(|_| Error::Unspecified)
    }

    fn mc_to_id(
        &self,
        id: &uuid::Uuid,
    ) -> impl Future<Output = super::Result<Option<uuid::Uuid>>> + Send {
        sqlx::query_scalar!("SELECT user_id FROM users where mc_id = $1", id)
            .fetch_optional(&self.pool)
            .map_err(|_| Error::Unspecified)
    }

    fn user_info(
        &self,
        id: &uuid::Uuid,
    ) -> impl Future<Output = super::Result<Option<crate::model::UserInfo>>> + Send {
        struct TmpUserInfo {
            /// The internal ID of the user
            pub user_id: Uuid,
            /// The Kromer balance of this user
            pub balance: Decimal,
            pub created_at: DateTime<Utc>,
            pub mc_id: Option<Uuid>,
            pub disc_id: Option<i64>,
        }

        sqlx::query_as!(
            TmpUserInfo,
            "SELECT user_id, balance, created_at, mc_id, disc_id FROM users WHERE user_id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .map(|res| match res {
            Ok(Some(u)) => {
                let info = UserInfo {
                    id: u.user_id,
                    balance: u.balance,
                    created_at: u.created_at,
                    mc_id: u.mc_id,
                    disc_id: u.disc_id.map(|v| {
                        let tmp: u64 = v.try_into().expect("Enforced by DB");

                        NonZeroU64::try_from(tmp).expect("Enforced by DB")
                    }),
                };

                Ok(Some(info))
            }
            Ok(None) => Ok(None),
            Err(_) => Err(Error::Unspecified),
        })
    }

    fn register_user(
        &self,
        disc_id: Option<i64>,
        mc_id: Option<&uuid::Uuid>,
    ) -> impl Future<Output = super::Result<uuid::Uuid>> + Send {
        sqlx::query_scalar!(
            "INSERT INTO users (disc_id, mc_id) VALUES ($1, $2) RETURNING user_id",
            disc_id,
            mc_id
        )
        .fetch_one(&self.pool)
        .map_err(|err| {
            if let Some(dberr) = err.as_database_error()
                && dberr.is_unique_violation()
            {
                Error::AlreadyLinked
            } else {
                Error::Unspecified
            }
        })
    }

    fn get_holdings(
        &self,
        id: &uuid::Uuid,
        page: &Pager,
    ) -> impl Future<Output = super::Result<Option<(Vec<(Ticker, u32)>, usize)>>> + Send {
        struct StockValues {
            pub ticker: String,
            pub shares: i32,
        }
        async move {
            let res = sqlx::query_as!(
                StockValues,
                "SELECT ticker, shares FROM holdings WHERE user_id = $1 ORDER BY ticker LIMIT $2 OFFSET $3",
                id,
                page.limit(),
                page.offset()
            )
            .fetch_all(&self.pool)
            .await
            .map_or_else(
                |err| match err {
                    sqlx::Error::RowNotFound => Ok(None),
                    _ => Err(Error::Unspecified),
                },
                |v| Ok(Some(v)),
            )?.unwrap_or_default();

            let res: Vec<_> = res
                .into_iter()
                .filter_map(|v| {
                    let ticker = Ticker::try_from(v.ticker.as_str());

                    match ticker {
                        Ok(ticker) => Some((ticker, v.shares.try_into().expect("Always works"))),
                        Err(_) => None,
                    }
                })
                .collect();

            let mut num =
                sqlx::query_scalar!("SELECT COUNT(*) FROM holdings WHERE user_id = $1", id)
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|_| Error::Unspecified)?
                    .unwrap_or_default();

            let len: i64 = res
                .len()
                .try_into()
                .expect("Database ensures that this is valid");

            num -= i64::max(0, page.offset() + len);

            Ok(Some((
                res,
                num.try_into().expect("Will always be positive"),
            )))
        }
    }

    // fn list_stocks(
    //     &self,
    //     page: Option<&crate::model::Pager>,
    // ) -> impl Future<Output = super::Result<(Vec<(Ticker, rust_decimal::Decimal, u32)>, usize)>> + Send
    // {
    //     unimplemented!()
    // }
}
