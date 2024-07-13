/******************************************************************************\
    sma-proto - A SMA Speedwire protocol library
    Copyright (C) 2024 Max Maisel

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
\******************************************************************************/
#[cfg(not(feature = "std"))]
use core::{fmt::Debug, prelude::rust_2021::derive};

/// Errors returned from SMA speedwire protocol processing.
#[derive(Clone, Debug)]
pub enum Error {
    /// The provided buffer is too small.
    BufferTooSmall { size: usize, expected: usize },
}

/// A specialized Result type for SMA speedwire operations.
pub type Result<T> = core::result::Result<T, Error>;
