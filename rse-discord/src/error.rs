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

use snafu::Snafu;

/// Poise result type
pub type DiscResult = Result<(), Error>;

/// Errors emitted by the discord integration. Need to sanitize this so it can be exposed back to
/// Discord users
#[derive(Debug, Snafu)]
#[allow(missing_docs)]
pub enum Error {
    /// Errors emitted by internal errors
    #[snafu(transparent)]
    ServiceError { source: rse_core::error::Error },

    /// Errors emitted by poise itself
    #[snafu(transparent)]
    PoiseError {
        source: poise::serenity_prelude::Error,
    },
}
