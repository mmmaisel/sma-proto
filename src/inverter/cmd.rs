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

use byteorder_cursor::{BigEndian, Cursor};

use super::{Result, SmaSerde};

/// A speedwire command word consisting of an opcode and a channel.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct SmaCmdWord {
    /// Channel number.
    pub channel: u8,
    /// 24bit command ID.
    pub opcode: u32,
}

impl SmaCmdWord {
    /// Serialized length of the command word.
    const LENGTH: usize = 4;
}

impl SmaSerde for SmaCmdWord {
    fn serialize(&self, buffer: &mut Cursor<&mut [u8]>) -> Result<()> {
        buffer.check_remaining(Self::LENGTH)?;
        buffer.write_u8(self.channel);
        buffer.write_u24::<BigEndian>(self.opcode);

        Ok(())
    }

    fn deserialize(buffer: &mut Cursor<&[u8]>) -> Result<Self> {
        buffer.check_remaining(Self::LENGTH)?;

        let channel = buffer.read_u8();
        let opcode = buffer.read_u24::<BigEndian>();

        Ok(Self { channel, opcode })
    }
}
