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
    Cursor, Error, Result, SmaCmdWord, SmaEndpoint, SmaInvCounter,
    SmaInvHeader, SmaPacketFooter, SmaPacketHeader, SmaSerde,
};
use byteorder::LittleEndian;
#[cfg(not(feature = "std"))]
use core::{
    clone::Clone,
    cmp::{Eq, PartialEq},
    fmt::Debug,
    prelude::rust_2021::derive,
    result::Result::{Err, Ok},
};

/// A logical SMA inverter logout message.
/// This message has no response.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SmaInvLogout {
    /// Destination application/device address.
    pub dst: SmaEndpoint,
    /// Source application/device address.
    pub src: SmaEndpoint,
    /// Non-zero in case of errors.
    pub error_code: u16,
    /// Packet counters.
    pub counters: SmaInvCounter,
}

impl SmaSerde for SmaInvLogout {
    fn serialize(&self, buffer: &mut Cursor<&mut [u8]>) -> Result<()> {
        buffer.check_remaining(Self::LENGTH)?;

        let data_len =
            Self::LENGTH - SmaPacketHeader::LENGTH - SmaPacketFooter::LENGTH;
        let header = SmaPacketHeader {
            data_len,
            protocol: SmaPacketHeader::SMA_PROTOCOL_INV,
        };

        let inv_header = SmaInvHeader {
            wordcount: (data_len / 4) as u8,
            class: 0xA0,
            dst: self.dst.clone(),
            dst_ctrl: 3,
            src: self.src.clone(),
            src_ctrl: 3,
            error_code: self.error_code,
            counters: self.counters.clone(),
            cmd: SmaCmdWord {
                channel: 0x0E,
                opcode: Self::OPCODE,
            },
        };

        header.serialize(buffer)?;
        inv_header.serialize(buffer)?;
        buffer.write_u32::<LittleEndian>(0xFFFFFFFF);
        SmaPacketFooter::default().serialize(buffer)?;

        Ok(())
    }

    fn deserialize(buffer: &mut Cursor<&[u8]>) -> Result<Self> {
        buffer.check_remaining(Self::LENGTH)?;

        let header = SmaPacketHeader::deserialize(buffer)?;
        header.check_protocol(SmaPacketHeader::SMA_PROTOCOL_INV)?;
        buffer.check_remaining(header.data_len)?;

        let inv_header = SmaInvHeader::deserialize(buffer)?;
        inv_header.check_wordcount(header.data_len)?;
        inv_header.check_class(0xA0)?;
        inv_header.check_opcode(Self::OPCODE)?;

        let padding = buffer.read_u32::<LittleEndian>();
        if padding != 0xFFFFFFFF {
            return Err(Error::InvalidPadding { padding });
        }

        SmaPacketFooter::deserialize(buffer)?;

        Ok(Self {
            src: inv_header.src,
            dst: inv_header.dst,
            error_code: inv_header.error_code,
            counters: inv_header.counters,
        })
    }
}

impl SmaInvLogout {
    pub const OPCODE: u32 = 0x01FDFF;
    pub const LENGTH: usize = SmaPacketHeader::LENGTH
        + SmaInvHeader::LENGTH
        + 4
        + SmaPacketFooter::LENGTH;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_inv_logout_serialization() {
        let cmd = SmaInvLogout {
            src: SmaEndpoint::dummy(),
            dst: SmaEndpoint {
                susy_id: 0x5678,
                serial: 0xABCDABCE,
            },
            counters: SmaInvCounter {
                packet_id: 1,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut buffer = [0u8; SmaInvLogout::LENGTH];
        let mut cursor = Cursor::new(&mut buffer[..]);

        if let Err(e) = cmd.serialize(&mut cursor) {
            panic!("SmaInvLogout serialization failed: {e:?}");
        }

        #[rustfmt::skip]
        let expected = [
            0x53, 0x4D, 0x41, 0x00, 0x00, 0x04, 0x02, 0xA0,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x22, 0x00, 0x10,
            0x60, 0x65,
            0x08, 0xA0,
            0x56, 0x78, 0xAB, 0xCD, 0xAB, 0xCE, 0x00, 0x03,
            0xDE, 0xAD, 0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x03,
            0x00, 0x00, 0x00, 0x00, 0x01, 0x80,
            0x0E, 0x01, 0xFD, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
            0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(SmaInvLogout::LENGTH, cursor.position());
        assert_eq!(expected, buffer);
    }

    #[test]
    fn test_sma_inv_logout_deserialization() {
        #[rustfmt::skip]
        let serialized = [
            0x53, 0x4D, 0x41, 0x00, 0x00, 0x04, 0x02, 0xA0,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x22, 0x00, 0x10,
            0x60, 0x65,
            0x08, 0xA0,
            0x56, 0x78, 0xAB, 0xCD, 0xAB, 0xCE, 0x00, 0x03,
            0xDE, 0xAD, 0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x03,
            0x00, 0x00, 0x00, 0x00, 0x01, 0x80,
            0x0E, 0x01, 0xFD, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
            0x00, 0x00, 0x00, 0x00,
        ];

        let expected = SmaInvLogout {
            src: SmaEndpoint::dummy(),
            dst: SmaEndpoint {
                susy_id: 0x5678,
                serial: 0xABCDABCE,
            },
            counters: SmaInvCounter {
                packet_id: 1,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut cursor = Cursor::new(&serialized[..]);
        match SmaInvLogout::deserialize(&mut cursor) {
            Err(e) => panic!("SmaInvLogout deserialization failed: {e:?}"),
            Ok(cmd) => {
                assert_eq!(expected, cmd);
                assert_eq!(SmaInvLogout::LENGTH, cursor.position());
            }
        }
    }
}
