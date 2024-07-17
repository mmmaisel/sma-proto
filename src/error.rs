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
    /// The wordcount field in the inverter sub-protocol header data length
    /// is invalid.
    InvalidWordcount { wordcount: u8 },
    /// The class field of this message has an unsupported value.
    UnsupportedCommandClass { class: u8 },
    /// The opcode of this message has an unsupported value.
    UnsupportedOpcode { opcode: u32 },
    /// The payload of a packet exceeds the maximum supported length.
    PayloadTooLarge { len: usize },
}

#[cfg(feature = "std")]
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::BufferTooSmall { size, expected } => {
                write!(
                    f,
                    "The supplied buffer is to small. \
                    Got {size}, expected at least {expected}"
                )
            }
            Self::BufferNotConsumed { trailing } => {
                write!(
                    f,
                    "The supplied buffer contained {trailing} trailing bytes"
                )
            }
            Self::InvalidFourCC { fourcc } => {
                write!(f, "Found invalid FOURCC value {fourcc:X}")
            }
            Self::InvalidStartTagLen { len } => {
                write!(f, "Found invalid start tag length {len}")
            }
            Self::InvalidStartTag { tag } => {
                write!(f, "Found invalid start tag value {tag:X}")
            }
            Self::InvalidGroup { group } => {
                write!(f, "Found invalid group {group:X}")
            }
            Self::UnsupportedVersion { version } => {
                write!(f, "Unsupported SMA protocol version {version}")
            }
            Self::UnsupportedProtocol { protocol } => {
                write!(f, "Unsupported SMA sub-protocol {protocol:X}")
            }
            Self::InvalidPadding { padding } => {
                write!(f, "Found non-zero padding value {padding:X}")
            }
            Self::UnsupportedObisId { id } => {
                write!(f, "Unsupported OBIS ID {id:X}")
            }
            Self::InvalidWordcount { wordcount } => {
                write!(
                    f,
                    "The word count {wordcount} in the protocol header \
                    is invalid"
                )
            }
            Self::UnsupportedCommandClass { class } => {
                write!(f, "Found unsupported command class {class:X}")
            }
            Self::UnsupportedOpcode { opcode } => {
                write!(f, "Found unsupported opcode {opcode:X}")
            }
            Self::PayloadTooLarge { len } => {
                write!(
                    f,
                    "The messages payload length {len} exceeds \
                    the supported maximum"
                )
            }
        }
    }
}

/// A specialized Result type for SMA speedwire operations.
pub type Result<T> = core::result::Result<T, Error>;
