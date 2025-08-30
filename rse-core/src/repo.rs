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

//! Abstract implementation details for the backing stock repository

use crate::model::{Pager, UserInfo, ticker::Ticker};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use snafu::Snafu;
use uuid::Uuid;

pub use pg::PgPort;
mod pg;

#[allow(missing_docs)]
pub type Result<T> = std::result::Result<T, Error>;

/// Common errors thrown when interfacing with a [`StockRepository`]
#[derive(Debug, Snafu, Clone, Copy, PartialEq, Eq)]
#[snafu(visibility(pub(crate)))]
#[allow(missing_docs)]
pub enum Error {
    /// Could not find the account linked to a given UUID
    #[snafu(display(r#"Could not find an account with UUID \"{id}\"#))]
    AccountNotFound { id: Uuid },
    /// Occurs when trying to create an account and and is passed a Discord snowflake or Minecraft
    /// UUID that is already linked to an account. Does not specify which as the given call site
    /// should be able to determine this.
    #[snafu(display("An account is already linked to this"))]
    AlreadyLinked,
    /// An underlying error that either do not know, or cannot handle
    #[snafu(display("An unspecified DB error occurred"))]
    Unspecified,
}

/// A port handling all the logic for storing and querying our backing data store.
pub trait StockRepository: 'static + Clone + Send + Sync {
    /// Checks if a user exists
    ///
    /// # Errors
    /// * [`Unspecified`](Error::Unspecified) - An issue with the underlying repository
    fn user_exists(&self, id: &Uuid) -> impl Future<Output = Result<bool>> + Send;

    /// Checks if stock exists
    ///
    /// # Errors
    /// * [`Unspecified`](Error::Unspecified) - An issue with the underlying repository
    fn stock_exists(&self, stock: &Ticker) -> impl Future<Output = Result<bool>> + Send;

    /// Takes a Discord snowflake and returns the UUID of the account its linked to if it exists.
    ///
    /// # Errors
    /// * [`Unspecified`](Error::Unspecified) - An issue with the underlying repository
    fn discord_to_id(&self, id: i64) -> impl Future<Output = Result<Option<Uuid>>> + Send;

    /// Takes a Minecraft UUID and returns the UUID of the account its linked to if it exists.
    ///
    /// # Errors
    /// * [`Unspecified`](Error::Unspecified) - An issue with the underlying repository
    fn mc_to_id(&self, id: &Uuid) -> impl Future<Output = Result<Option<Uuid>>> + Send;

    /// Retrieves user info given an ID, returning it if it exists
    ///
    /// # Errors
    /// * [`Unspecified`](Error::Unspecified) - An issue with the underlying repository
    fn user_info(&self, id: &Uuid) -> impl Future<Output = Result<Option<UserInfo>>> + Send;

    /// Registers a user, returning the UUID of the new user.
    ///
    /// # Arguments
    /// At least one of `disc_id` and `mc_id` should be [Some], but never both for external
    /// endpoints.
    /// * `disc_id` - The Discord snowflake of the new user
    /// * `mc_id`- The Minecraft UUID of the new user
    ///
    /// # Errors
    /// * [`AlreadyLinked`](Error::AlreadyLinked) - The passed in ID is already linked to an
    ///   account
    /// * [`Unspecified`](Error::Unspecified) - An issue with the underlying repository
    fn register_user(
        &self,
        disc_id: Option<i64>,
        mc_id: Option<&Uuid>,
    ) -> impl Future<Output = Result<Uuid>> + Send;

    /// Lists a user's holdings in a paginated way, as well as the total number of entries.
    ///
    /// # Errors
    /// * [`Unspecified`](Error::Unspecified) - An issue with the underlying repository
    #[allow(clippy::type_complexity)]
    fn get_holdings(
        &self,
        id: &Uuid,
        page: &Pager,
    ) -> impl Future<Output = Result<Option<(Vec<(Ticker, u32)>, i64)>>> + Send;

    /// Lists all stocks
    ///
    /// # Errors
    /// * [`Unspecified`](Error::Unspecified) - An issue with the underlying repository
    #[allow(clippy::type_complexity)]
    fn list_stocks(
        &self,
        page: &Pager,
    ) -> impl Future<Output = Result<Option<(Vec<(Ticker, u32, Decimal, DateTime<Utc>)>, i64)>>> + Send;
}
