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
use byteorder_cursor::{BigEndian, Cursor};

use super::{Error, Result, SmaSerde};

/// A tuple consisting of an OBIS ID and its value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObisValue {
    /// 32bit encoded OBIS number.
    pub id: u32,
    /// Value of up to 64bit.
    /// The actual size and unit is determined by the OBIS ID.
    pub value: u64,
}

impl ObisValue {
    /// Minimum serialized length of one OBIS value.
    pub const LENGTH_MIN: usize = 8;
    /// Maximum serialized length of one OBIS value.
    pub const LENGTH_MAX: usize = 12;

    /// Serialized length of this OBIS value.
    pub fn serialized_len(&self) -> usize {
        if self.id == 0x90000000 || self.id & 0xFF00 == 0x0400 {
            8
        } else if self.id & 0xFF00 == 0x0800 {
            12
        } else {
            0
        }
    }

    /// Checks is the OBIS ID is valid and supported.
    pub fn validate(&self) -> Result<()> {
        if self.id == 0x90000000
            || self.id & 0xFF00 == 0x0400
            || self.id & 0xFF00 == 0x0800
        {
            Ok(())
        } else {
            Err(Error::UnsupportedObisId { id: self.id })
        }
    }
}

impl SmaSerde for ObisValue {
    fn serialize(&self, buffer: &mut Cursor<&mut [u8]>) -> Result<()> {
        self.validate()?;
        buffer.check_remaining(self.serialized_len())?;

        buffer.write_u32::<BigEndian>(self.id);
        if self.id == 0x90000000 || self.id & 0xFF00 == 0x0400 {
            buffer.write_u32::<BigEndian>(self.value as u32);
        } else if self.id & 0xFF00 == 0x0800 {
            buffer.write_u64::<BigEndian>(self.value);
        }

        Ok(())
    }

    fn deserialize(buffer: &mut Cursor<&[u8]>) -> Result<Self> {
        buffer.check_remaining(Self::LENGTH_MIN)?;

        let id = buffer.read_u32::<BigEndian>();
        let value = if id == 0x90000000 || id & 0xFF00 == 0x0400 {
            buffer.read_u32::<BigEndian>() as u64
        } else if id & 0xFF00 == 0x0800 {
            buffer.check_remaining(8)?;
            buffer.read_u64::<BigEndian>()
        } else {
            return Err(Error::UnsupportedObisId { id });
        };

        let obj = Self { id, value };
        obj.validate()?;

        Ok(obj)
    }
}
