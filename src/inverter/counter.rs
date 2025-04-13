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
use core::{
    clone::Clone,
    cmp::{Eq, PartialEq},
    fmt::Debug,
    prelude::rust_2021::derive,
    result::Result::Ok,
};

use byteorder_cursor::{Cursor, LittleEndian};

use super::{Result, SmaSerde};

/// SMA inverter sub-protocol packet and fragment counter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SmaInvCounter {
    /// Decrementing packet fragment counter.
    pub fragment_id: u16,
    /// Incrementing packet counter.
    pub packet_id: u16,
    /// Indicates the first fragment in a sequence.
    pub first_fragment: bool,
}

impl Default for SmaInvCounter {
    fn default() -> Self {
        Self {
            fragment_id: 0,
            packet_id: 0,
            first_fragment: true,
        }
    }
}

impl SmaInvCounter {
    pub const LENGTH: usize = 4;
    pub const FIRST_FRAGMENT_BIT: u16 = 0x8000;
}

impl SmaSerde for SmaInvCounter {
    fn serialize(&self, buffer: &mut Cursor<&mut [u8]>) -> Result<()> {
        buffer.check_remaining(Self::LENGTH)?;

        let packet_id = if self.first_fragment {
            self.packet_id | Self::FIRST_FRAGMENT_BIT
        } else {
            self.packet_id
        };

        buffer.write_u16::<LittleEndian>(self.fragment_id);
        buffer.write_u16::<LittleEndian>(packet_id);

        Ok(())
    }

    fn deserialize(buffer: &mut Cursor<&[u8]>) -> Result<Self> {
        buffer.check_remaining(Self::LENGTH)?;

        let fragment_id = buffer.read_u16::<LittleEndian>();
        let raw_packet_id = buffer.read_u16::<LittleEndian>();
        let (packet_id, first_fragment) =
            if (raw_packet_id & Self::FIRST_FRAGMENT_BIT) != 0 {
                (raw_packet_id & !Self::FIRST_FRAGMENT_BIT, true)
            } else {
                (raw_packet_id, false)
            };

        Ok(Self {
            fragment_id,
            packet_id,
            first_fragment,
        })
    }
}
