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

//! Error types for the core `RSE` stock service

use std::num::NonZeroU64;

use snafu::Snafu;

#[allow(missing_docs)]
pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned to ingress ports. Should not be directly returned to end users, but rather
/// transformed into an appropriate error that does not expose any internal details of our API.
#[derive(Debug, Snafu, Clone, Copy)]
#[snafu(visibility(pub(crate)))]
#[allow(missing_docs, variant_size_differences)]
pub enum Error {
    /// When a discord snowflake does not fit inside an i64
    #[snafu(display(r#""{flake}" is not a valid Discord Snowflake"#))]
    InvalidSnowflake { flake: NonZeroU64 },
    /// An issue with the underlying Database that we either do not know or can't handle
    #[snafu(display("Encountered an internal error"))]
    DatabaseError { source: crate::repo::Error },
    /// When trying to register and there is already an account linked to the provided ID
    #[snafu(display("The provided account already exists"))]
    AccountExists,
    /// Could not find an account linked to a given ID. Meant for use from external services such
    /// as Discord or `Chatbox`.
    #[snafu(display("There is no account linked to passed ID"))]
    UserNotFound,
}

impl From<crate::repo::Error> for Error {
    fn from(value: crate::repo::Error) -> Self {
        use crate::repo::Error as RepError;
        match value {
            RepError::AlreadyLinked => Self::AccountExists,
            _ => Self::DatabaseError { source: value },
        }
    }
}
