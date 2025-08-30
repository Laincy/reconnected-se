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

//! Types that model our stock domain

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::num::NonZeroU64;
use uuid::Uuid;

pub mod ticker;

/// Information about a given user
#[derive(Debug, Clone, Copy)]
pub struct UserInfo {
    /// The internal ID of the user
    pub id: Uuid,
    /// The Kromer balance of this user
    pub balance: Decimal,
    /// When the user was created
    pub created_at: DateTime<Utc>,
    /// The linked Minecraft ID
    pub mc_id: Option<Uuid>,
    /// The linked Discord ID
    pub disc_id: Option<NonZeroU64>,
}

/// A paginated request helper
#[derive(Debug, Clone, Copy)]
pub struct Pager {
    offset: i64,
    limit: i64,
}

impl Pager {
    /// Creates a new [`Pager`]
    #[must_use]
    pub fn new(offset: i64, limit: i64) -> Self {
        Self {
            offset,
            limit: limit.max(1),
        }
    }

    /// A `getter` for the Pager's offset
    #[must_use]
    pub const fn offset(&self) -> i64 {
        self.offset
    }

    /// A `getter` for the Pager's limit
    #[must_use]
    pub const fn limit(&self) -> i64 {
        self.limit
    }
    /// Increments the Pager's offset
    pub const fn add_offset(&mut self, v: i64) {
        self.offset += v;
    }

    /// Increments the Pager's limit
    pub const fn add_limit(&mut self, v: i64) {
        self.limit += v;
    }

    /// Sets the Pager's offset
    pub const fn set_offset(&mut self, v: i64) {
        self.offset = v;
    }

    /// Sets the Pager's limit
    pub const fn set_limit(&mut self, v: i64) {
        self.limit = v;
    }
}
