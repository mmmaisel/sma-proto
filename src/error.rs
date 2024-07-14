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
    /// The provided buffer contained unexpected trailing bytes and
    /// was not completely deserialized.
    BufferNotConsumed { trailing: usize },
    /// The processed packet starts with an invalid SMA FOURCC value.
    InvalidFourCC { fourcc: u32 },
    /// The packet header length is incorrect.
    InvalidStartTagLen { len: u16 },
    /// The start tag value in the common packet header is invalid.
    InvalidStartTag { tag: u16 },
    /// The group value in the common packet header is invalid.
    InvalidGroup { group: u32 },
    /// The protocol version as indicated in the common packet header
    /// is unsupported.
    UnsupportedVersion { version: u16 },
    /// The sub-protocol type as indicated in the common packet header
    /// is unsupported.
    UnsupportedProtocol { protocol: u16 },
    /// The padding bytes are not all zero.
    InvalidPadding { padding: u32 },
    /// The OBIS ID encountered is unsupported.
    UnsupportedObisId { id: u32 },
    /// The payload of a packet exceeds the maximum supported length.
    PayloadTooLarge { len: usize },
}

/// A specialized Result type for SMA speedwire operations.
pub type Result<T> = core::result::Result<T, Error>;
