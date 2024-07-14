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
use super::{Cursor, Result, SmaEndpoint, SmaSerde};
use byteorder::BigEndian;
#[cfg(not(feature = "std"))]
use core::{
    clone::Clone,
    cmp::{Eq, PartialEq},
    fmt::Debug,
    prelude::rust_2021::derive,
    result::Result::Ok,
};

/// SMA energymeter sub-protocol header.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SmaEmHeader {
    /// Source endpoint address.
    pub src: SmaEndpoint,
    /// Overflowing timestamp in milliseconds.
    pub timestamp_ms: u32,
}

impl SmaEmHeader {
    /// Serialized length of the energymeter sub-protocol header.
    pub const LENGTH: usize = 10;
}

impl SmaSerde for SmaEmHeader {
    fn serialize(&self, buffer: &mut Cursor<&mut [u8]>) -> Result<()> {
        buffer.check_remaining(Self::LENGTH)?;

        self.src.serialize(buffer)?;
        buffer.write_u32::<BigEndian>(self.timestamp_ms);

        Ok(())
    }

    fn deserialize(buffer: &mut Cursor<&[u8]>) -> Result<Self> {
        buffer.check_remaining(Self::LENGTH)?;

        let src = SmaEndpoint::deserialize(buffer)?;
        let timestamp_ms = buffer.read_u32::<BigEndian>();

        Ok(Self { src, timestamp_ms })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_em_header_serialization() {
        let header = SmaEmHeader {
            src: SmaEndpoint {
                susy_id: 0x1234,
                serial: 0xDEADBEEF,
            },
            timestamp_ms: 1_000_000,
        };
        let mut buffer = [0u8; SmaEmHeader::LENGTH];
        let mut cursor = Cursor::new(&mut buffer[..]);

        if let Err(e) = header.serialize(&mut cursor) {
            panic!("SmaEmHeader serialization failed: {e:?}");
        }

        #[rustfmt::skip]
        let expected = [
            0x12, 0x34,
            0xDE, 0xAD, 0xBE, 0xEF,
            0x00, 0x0F, 0x42, 0x40,
        ];
        assert_eq!(SmaEmHeader::LENGTH, cursor.position());
        assert_eq!(expected, buffer);
    }

    #[test]
    fn test_sma_em_header_deserialization() {
        #[rustfmt::skip]
        let serialized = [
            0x12, 0x34,
            0xDE, 0xAD, 0xBE, 0xEF,
            0x00, 0x0F, 0x42, 0x40,
        ];

        let expected = SmaEmHeader {
            src: SmaEndpoint {
                susy_id: 0x1234,
                serial: 0xDEADBEEF,
            },
            timestamp_ms: 1_000_000,
        };

        let mut cursor = Cursor::new(&serialized[..]);
        match SmaEmHeader::deserialize(&mut cursor) {
            Err(e) => panic!("SmaEmHeader deserialization failed: {e:?}"),
            Ok(header) => {
                assert_eq!(expected, header);
                assert_eq!(SmaEmHeader::LENGTH, cursor.position());
            }
        };
    }
}
