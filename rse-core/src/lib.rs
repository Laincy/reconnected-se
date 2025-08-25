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

//! The core of our system. Includes a generic stock service type which abstracts over our
//! underlying data stores and notifiers.

use std::num::NonZeroI64;

use crate::{
    error::UserNotFoundSnafu,
    model::{Pager, UserInfo, ticker::Ticker},
    repo::StockRepository,
};
use error::Result;
use snafu::OptionExt;
use uuid::Uuid;

#[allow(unused_imports)] // Used for docs
use error::Error;

pub mod error;
pub mod model;
pub mod repo;

/// A cheaply cloneable service managing our core business logic
#[derive(Debug, Clone)]
pub struct Service<R: StockRepository> {
    repo: R,
}

impl<R: StockRepository> Service<R> {
    /// Create a new instance of [`Service`]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Gets the UUID of an account from its linked Discord snowflake.
    ///
    /// # Errors
    /// * [`AccountNotFound`](Error::AccountNotFound) - There is no account linked with the
    ///   provided Snowflake
    /// * [`DatabaseError`](Error::DatabaseError) - An issue with the underlying data store
    pub async fn disc_to_id(&self, id: NonZeroI64) -> Result<Uuid> {
        self.repo
            .discord_to_id(id.get())
            .await?
            .context(UserNotFoundSnafu)
    }

    /// Gets the UUID of an account from its linked Minecraft UUID
    ///
    /// # Errors
    /// * [`AccountNotFound`](Error::AccountNotFound) - There is no account linked with the
    ///   provided Minecraft account
    /// * [`DatabaseError`](Error::DatabaseError) - An issue with the underlying data store
    pub async fn mc_to_id(&self, id: &Uuid) -> Result<Uuid> {
        self.repo.mc_to_id(id).await?.context(UserNotFoundSnafu)
    }

    /// Registers an account, linking it to a given user.
    ///
    /// # Arguments
    /// These arguments should have one [Some] and one [None]. Anything else will panic in debug
    /// mode and cause potential issues in release.
    /// * `disc_id` - The Discord snowflake to link to
    /// * `mc_id` - The Minecraft UUID to link to
    ///
    /// # Errors
    /// * [`AccountExists`](Error::AccountExists) - The provided ID is already linked to an account
    /// * [`DatabaseError`](Error::DatabaseError) - An issue with the underlying data store
    pub async fn register_account(
        &self,
        disc_id: Option<NonZeroI64>,
        mc_id: Option<&Uuid>,
    ) -> Result<Uuid> {
        debug_assert_ne!(
            disc_id.is_some(),
            mc_id.is_some(),
            "Only one of these values should ever be Some"
        );

        Ok(self
            .repo
            .register_user(disc_id.map(NonZeroI64::get), mc_id)
            .await?)
    }

    /// Gets information about a given account
    ///
    /// # Errors
    /// * [`UserNotFound`](Error::UserNotFound) - There is no account linked to ID
    /// * [`DatabaseError`](Error::DatabaseError) - An issue with the underlying data store
    pub async fn get_account_info(&self, id: &Uuid) -> Result<UserInfo> {
        self.repo.user_info(id).await?.context(UserNotFoundSnafu)
    }

    /// Lists all of a user's holdings in a paginated way. Also returns the number of remaining
    /// entries.
    ///
    /// # Errors
    /// * [`UserNotFound`](Error::UserNotFound) - There is no account linked to ID
    /// * [`DatabaseError`](Error::DatabaseError) - An issue with the underlying data store
    pub async fn get_holdings(
        &self,
        id: &Uuid,
        page: &Pager,
    ) -> Result<(Vec<(Ticker, u32)>, usize)> {
        self.repo
            .get_holdings(id, page)
            .await?
            .context(UserNotFoundSnafu)
    }

    // /// Lists all stocks on the market, returning their ticker, most recent sell price, and number
    // /// of shares. Also returns the number of stocks remaining if this is called with a pager.
    // ///
    // /// # Errors
    // /// * [`DatabaseError`](Error::DatabaseError) - An issue with the underlying data store
    // pub async fn list_stocks(
    //     &self,
    //     page: Option<&Pager>,
    // ) -> Result<(Vec<(Ticker, Decimal, u32)>, usize)> {
    //     Ok(self.repo.list_stocks(page).await?)
    // }
}
