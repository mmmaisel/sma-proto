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

//! Module for handling the SMA speedwire inverter sub protocol.

use super::{
    Cursor, Error, Result, SmaEndpoint, SmaPacketFooter, SmaPacketHeader,
    SmaSerde,
};

mod cmd;
mod counter;
mod get_day_data;
mod header;
mod identify;
mod login;
mod logout;
mod meter;

use cmd::SmaCmdWord;
use counter::SmaInvCounter;
use header::SmaInvHeader;

pub use get_day_data::SmaInvGetDayData;
pub use identify::SmaInvIdentify;
pub use login::{InvalidPasswordError, SmaInvLogin};
pub use logout::SmaInvLogout;
pub use meter::SmaInvMeterValue;
