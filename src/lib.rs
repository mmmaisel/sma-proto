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
#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
#![forbid(unsafe_code)]

mod any;
mod container;
mod error;
mod packet;

#[cfg(feature = "client")]
pub mod client;
pub mod energymeter;
pub mod inverter;

pub use container::SmaContainer;
use packet::{SmaPacketFooter, SmaPacketHeader};

#[cfg(feature = "heapless")]
pub use any::AnySmaMessageHeapless;
#[cfg(feature = "std")]
pub use any::AnySmaMessageStd;
pub use any::{AnySmaMessage, AnySmaMessageBase};
pub use error::{Error, Result};
pub use packet::{SmaEndpoint, SmaSerde};
