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
use super::{Cursor, Result, SmaSerde};
use byteorder::LittleEndian;
#[cfg(not(feature = "std"))]
use core::{
    clone::Clone,
    cmp::{Eq, PartialEq},
    fmt::Debug,
    prelude::rust_2021::derive,
    result::Result::Ok,
};

/// Total inverter energy production at a given timestamp.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SmaInvMeterValue {
    /// Unix timestamp of the meter value.
    pub timestamp: u32,
    /// Total energy production in Wh.
    pub energy_wh: u64,
}

impl SmaInvMeterValue {
    pub const LENGTH: usize = 12;
}

impl SmaSerde for SmaInvMeterValue {
    fn serialize(&self, buffer: &mut Cursor<&mut [u8]>) -> Result<()> {
        buffer.check_remaining(Self::LENGTH)?;

        buffer.write_u32::<LittleEndian>(self.timestamp);
        buffer.write_u64::<LittleEndian>(self.energy_wh);

        Ok(())
    }

    fn deserialize(buffer: &mut Cursor<&[u8]>) -> Result<Self> {
        buffer.check_remaining(Self::LENGTH)?;

        let timestamp = buffer.read_u32::<LittleEndian>();
        let energy_wh = buffer.read_u64::<LittleEndian>();

        Ok(Self {
            timestamp,
            energy_wh,
        })
    }
}
