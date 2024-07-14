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
use super::{
    Cursor, Error, Result, SmaCmdWord, SmaEndpoint, SmaInvCounter, SmaSerde,
};
use byteorder::BigEndian;
#[cfg(not(feature = "std"))]
use core::{
    clone::Clone,
    cmp::{Eq, PartialEq},
    fmt::Debug,
    prelude::rust_2021::derive,
    result::Result::{Err, Ok},
};

/// SMA inverter sub-protocol header.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SmaInvHeader {
    /// Length of the sub-protocol section in 32bit words.
    pub wordcount: u8,
    /// Command class.
    pub class: u8,
    /// Destination application/device address.
    pub dst: SmaEndpoint,
    /// Command specific destination control word.
    pub dst_ctrl: u16,
    /// Source application/device address.
    pub src: SmaEndpoint,
    /// Command specific source control word.
    pub src_ctrl: u16,
    /// Non-zero in case of errors.
    pub error_code: u16,
    /// Packet and fragment counters.
    pub counters: SmaInvCounter,
    /// Command opcode and channel.
    pub cmd: SmaCmdWord,
}

impl SmaInvHeader {
    /// Serialized length of the inveter sub-protocol header.
    pub const LENGTH: usize = 28;
}

impl SmaSerde for SmaInvHeader {
    fn serialize(&self, buffer: &mut Cursor<&mut [u8]>) -> Result<()> {
        buffer.check_remaining(Self::LENGTH)?;

        buffer.write_u8(self.wordcount);
        buffer.write_u8(self.class);

        self.dst.serialize(buffer)?;
        buffer.write_u16::<BigEndian>(self.dst_ctrl);

        self.src.serialize(buffer)?;
        buffer.write_u16::<BigEndian>(self.src_ctrl);

        buffer.write_u16::<BigEndian>(self.error_code);
        self.counters.serialize(buffer)?;
        self.cmd.serialize(buffer)?;

        Ok(())
    }

    fn deserialize(buffer: &mut Cursor<&[u8]>) -> Result<Self> {
        buffer.check_remaining(Self::LENGTH)?;

        let wordcount = buffer.read_u8();
        let class = buffer.read_u8();

        let dst = SmaEndpoint::deserialize(buffer)?;
        let dst_ctrl = buffer.read_u16::<BigEndian>();

        let src = SmaEndpoint::deserialize(buffer)?;
        let src_ctrl = buffer.read_u16::<BigEndian>();

        let error_code = buffer.read_u16::<BigEndian>();
        let counters = SmaInvCounter::deserialize(buffer)?;
        let cmd = SmaCmdWord::deserialize(buffer)?;

        Ok(Self {
            wordcount,
            class,
            dst,
            dst_ctrl,
            src,
            src_ctrl,
            error_code,
            counters,
            cmd,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_inv_header_serialization() {
        let header = SmaInvHeader {
            wordcount: 16,
            class: 0xE0,
            dst: SmaEndpoint {
                susy_id: 0x5678,
                serial: 0xABCDABCE,
            },
            dst_ctrl: 0x33CC,
            src: SmaEndpoint {
                susy_id: 0x1234,
                serial: 0xDEADBEEF,
            },
            src_ctrl: 0x55AA,
            error_code: 0x1122,
            counters: SmaInvCounter {
                fragment_id: 10,
                packet_id: 5,
                first_fragment: false,
            },
            cmd: SmaCmdWord {
                channel: 0x10,
                opcode: 0x203040,
            },
        };
        let mut buffer = [0u8; SmaInvHeader::LENGTH];
        let mut cursor = Cursor::new(&mut buffer[..]);

        if let Err(e) = header.serialize(&mut cursor) {
            panic!("SmaInvHeader serialization failed: {e:?}");
        }

        #[rustfmt::skip]
        let expected = [
            0x10, 0xE0,
            0x56, 0x78, 0xAB, 0xCD, 0xAB, 0xCE, 0x33, 0xCC,
            0x12, 0x34, 0xDE, 0xAD, 0xBE, 0xEF, 0x55, 0xAA,
            0x11, 0x22, 0x0A, 0x00, 0x05, 0x00,
            0x10, 0x20, 0x30, 0x40,
        ];
        assert_eq!(SmaInvHeader::LENGTH, cursor.position());
        assert_eq!(expected, buffer);
    }

    #[test]
    fn test_sma_inv_header_deserialization() {
        #[rustfmt::skip]
        let serialized = [
            0x10, 0xE0,
            0x56, 0x78, 0xAB, 0xCD, 0xAB, 0xCE, 0x33, 0xCC,
            0x12, 0x34, 0xDE, 0xAD, 0xBE, 0xEF, 0x55, 0xAA,
            0x11, 0x22, 0x0A, 0x00, 0x05, 0x00,
            0x10, 0x20, 0x30, 0x40,
        ];

        let expected = SmaInvHeader {
            wordcount: 16,
            class: 0xE0,
            dst: SmaEndpoint {
                susy_id: 0x5678,
                serial: 0xABCDABCE,
            },
            dst_ctrl: 0x33CC,
            src: SmaEndpoint {
                susy_id: 0x1234,
                serial: 0xDEADBEEF,
            },
            src_ctrl: 0x55AA,
            error_code: 0x1122,
            counters: SmaInvCounter {
                fragment_id: 10,
                packet_id: 5,
                first_fragment: false,
            },
            cmd: SmaCmdWord {
                channel: 0x10,
                opcode: 0x203040,
            },
        };

        let mut cursor = Cursor::new(&serialized[..]);
        match SmaInvHeader::deserialize(&mut cursor) {
            Err(e) => panic!("SmaInvHeader deserialization failed: {e:?}"),
            Ok(header) => {
                assert_eq!(expected, header);
                assert_eq!(SmaInvHeader::LENGTH, cursor.position());
            }
        }
    }
}
