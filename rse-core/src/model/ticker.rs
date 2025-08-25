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

//! Parsing of stock tickers

use snafu::ensure;

/// A 3-5 character stock ticker consisting of uppercase ASCII letters
#[derive(Debug, Clone, Copy, Eq)]
pub struct Ticker([u8; 5], u8);

impl Ticker {
    /// Creates a new [`Ticker`].
    ///
    /// # Errors
    /// See [`ParseError`] for more information
    pub fn new(v: &[u8]) -> Result<Self, ParseError> {
        let len = v.len();
        ensure!((3..=5).contains(&len), InvalidLenSnafu);
        ensure!(v.iter().all(u8::is_ascii_alphabetic), InvalidCharsSnafu);
        let mut res = [0u8; 5];

        res[..len].copy_from_slice(v);
        res.make_ascii_uppercase();

        Ok(Self(
            res,
            len.try_into()
                .expect("This will always be a valid u8, we verified above"),
        ))
    }

    /// Gets the inner value as a string slice
    #[must_use]
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0[..self.1 as usize])
            .expect("Tickers being valid ASCII, and therefore UTF-8, is one of our invariants")
    }
}

impl std::fmt::Display for Ticker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl PartialEq for Ticker {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl std::hash::Hash for Ticker {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl TryFrom<&str> for Ticker {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value.as_bytes())
    }
}

impl TryFrom<&[u8]> for Ticker {
    type Error = ParseError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// Errors when parsing a value into a [`Ticker`]
#[derive(Debug, snafu::Snafu, Clone, Copy)]
pub enum ParseError {
    /// All characters must be ASCII letters
    #[snafu(display("All characters must be ASCII letters"))]
    InvalidChars,
    /// All tickers must be between 3 and 5 characters
    #[snafu(display("Length must be between 3 and 5 characters"))]
    InvalidLen,
}
