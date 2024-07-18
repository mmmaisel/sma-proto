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

//! High level tokio based SMA speedwire client.

use super::SmaEndpoint;

/// SMA client instance for communication with devices.
/// This object holds the network independent communication state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SmaClient {
    /// Client SMA endpoint ID.
    endpoint: SmaEndpoint,
    /// Current packet number.
    packet_id: u16,
}

impl SmaClient {
    /// Creates a new SmaClient with the given SmaEndpoint as source ID.
    pub fn new(endpoint: SmaEndpoint) -> Self {
        Self {
            endpoint,
            packet_id: 0,
        }
    }
}
