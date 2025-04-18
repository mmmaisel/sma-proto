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

//! Module for handling the SMA speedwire energy meter sub protocol.

use super::{
    Error, Result, SmaEndpoint, SmaPacketFooter, SmaPacketHeader, SmaSerde,
};

mod header;
mod message;
mod obis;

use header::SmaEmHeader;
#[cfg(feature = "heapless")]
pub use message::SmaEmMessageHeapless;
#[cfg(feature = "std")]
pub use message::SmaEmMessageStd;
pub use message::{SmaEmMessage, SmaEmMessageBase};
pub use obis::ObisValue;
