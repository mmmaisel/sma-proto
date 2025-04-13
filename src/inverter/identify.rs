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

use byteorder_cursor::Cursor;

use super::{
    Result, SmaCmdWord, SmaEndpoint, SmaInvCounter, SmaInvHeader,
    SmaPacketFooter, SmaPacketHeader, SmaSerde,
};

/// A logical SMA inverter identify message.
/// This message is sent to the broadcast serial/SUSy ID gets a response
/// with the corresponding source SMA endpoint.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SmaInvIdentify {
    /// Destination application/device address.
    pub dst: SmaEndpoint,
    /// Source application/device address.
    pub src: SmaEndpoint,
    /// Non-zero in case of errors.
    pub error_code: u16,
    /// Packet counters.
    pub counters: SmaInvCounter,
    /// Unknown identity binary data in response packet.
    pub identity: Option<[u8; Self::PAYLOAD_MAX]>,
}

impl SmaSerde for SmaInvIdentify {
    fn serialize(&self, buffer: &mut Cursor<&mut [u8]>) -> Result<()> {
        let data_len = if self.identity.is_some() {
            buffer.check_remaining(Self::LENGTH_MAX)?;
            Self::LENGTH_MAX - SmaPacketHeader::LENGTH - SmaPacketFooter::LENGTH
        } else {
            buffer.check_remaining(Self::LENGTH_MIN)?;
            Self::LENGTH_MIN - SmaPacketHeader::LENGTH - SmaPacketFooter::LENGTH
        };

        let header = SmaPacketHeader {
            data_len,
            protocol: SmaPacketHeader::SMA_PROTOCOL_INV,
        };

        let (dst_ctrl, channel) = if self.identity.is_some() {
            (0xC0, 1)
        } else {
            (0, 0)
        };

        let inv_header = SmaInvHeader {
            wordcount: (data_len / 4) as u8,
            class: 0xA0,
            dst: self.dst.clone(),
            dst_ctrl,
            src: self.src.clone(),
            error_code: self.error_code,
            counters: self.counters.clone(),
            cmd: SmaCmdWord {
                channel,
                opcode: Self::OPCODE,
            },
            ..Default::default()
        };

        header.serialize(buffer)?;
        inv_header.serialize(buffer)?;

        if let Some(identity) = self.identity {
            buffer.write_bytes(&identity);
        } else {
            buffer.write_bytes(&[0; Self::PAYLOAD_MIN]);
        }

        SmaPacketFooter::default().serialize(buffer)?;

        Ok(())
    }

    fn deserialize(buffer: &mut Cursor<&[u8]>) -> Result<Self> {
        buffer.check_remaining(Self::LENGTH_MIN)?;

        let header = SmaPacketHeader::deserialize(buffer)?;
        header.check_protocol(SmaPacketHeader::SMA_PROTOCOL_INV)?;
        buffer.check_remaining(header.data_len)?;

        let inv_header = SmaInvHeader::deserialize(buffer)?;
        inv_header.check_wordcount(header.data_len)?;
        inv_header.check_class(0xA0)?;
        inv_header.check_opcode(Self::OPCODE)?;

        let mut identity = [0; Self::PAYLOAD_MAX];
        let identity =
            if header.data_len - SmaInvHeader::LENGTH >= Self::PAYLOAD_MAX {
                buffer.read_bytes(&mut identity);
                Some(identity)
            } else {
                buffer.skip(Self::PAYLOAD_MIN);
                None
            };

        SmaPacketFooter::deserialize(buffer)?;

        Ok(Self {
            dst: inv_header.dst,
            src: inv_header.src,
            error_code: inv_header.error_code,
            counters: inv_header.counters,
            identity,
        })
    }
}

impl SmaInvIdentify {
    pub const OPCODE: u32 = 0x020000;
    pub const LENGTH_MIN: usize = SmaPacketHeader::LENGTH
        + SmaInvHeader::LENGTH
        + Self::PAYLOAD_MIN
        + SmaPacketFooter::LENGTH;
    pub const LENGTH_MAX: usize = SmaPacketHeader::LENGTH
        + SmaInvHeader::LENGTH
        + Self::PAYLOAD_MAX
        + SmaPacketFooter::LENGTH;
    pub const PAYLOAD_MIN: usize = 8;
    pub const PAYLOAD_MAX: usize = 48;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_inv_identify_serialization() {
        let cmd = SmaInvIdentify {
            dst: SmaEndpoint::broadcast(),
            src: SmaEndpoint {
                susy_id: 0xDEAD,
                serial: 0xDEADBEEF,
            },
            error_code: 0,
            counters: SmaInvCounter {
                packet_id: 0,
                ..Default::default()
            },
            identity: None,
        };

        let mut buffer = [0u8; SmaInvIdentify::LENGTH_MIN];
        let mut cursor = Cursor::new(&mut buffer[..]);

        if let Err(e) = cmd.serialize(&mut cursor) {
            panic!("SmaInvIdentify serialization failed: {e:?}");
        }

        #[rustfmt::skip]
        let expected = [
            0x53, 0x4D, 0x41, 0x00, 0x00, 0x04, 0x02, 0xA0,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x26, 0x00, 0x10,
            0x60, 0x65,
            0x09, 0xA0,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00,
            0xDE, 0xAD, 0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x80,
            0x00, 0x02, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(SmaInvIdentify::LENGTH_MIN, cursor.position());
        assert_eq!(expected, buffer);
    }

    #[test]
    fn test_sma_inv_identify_deserialization() {
        #[rustfmt::skip]
        let serialized = [
            0x53, 0x4D, 0x41, 0x00, 0x00, 0x04, 0x02, 0xA0,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x26, 0x00, 0x10,
            0x60, 0x65,
            0x09, 0xA0,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00,
            0xDE, 0xAD, 0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x80,
            0x00, 0x02, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let expected = SmaInvIdentify {
            dst: SmaEndpoint::broadcast(),
            src: SmaEndpoint {
                susy_id: 0xDEAD,
                serial: 0xDEADBEEF,
            },
            error_code: 0,
            counters: SmaInvCounter {
                packet_id: 0,
                ..Default::default()
            },
            identity: None,
        };

        let mut cursor = Cursor::new(&serialized[..]);
        match SmaInvIdentify::deserialize(&mut cursor) {
            Err(e) => panic!("SmaInvIdentify deserialization failed: {e:?}"),
            Ok(cmd) => {
                assert_eq!(expected, cmd);
                assert_eq!(SmaInvIdentify::LENGTH_MIN, cursor.position());
            }
        }
    }

    #[test]
    fn test_sma_inv_identify_response_deserialization() {
        #[rustfmt::skip]
        let serialized = [
            0x53, 0x4D, 0x41, 0x00, 0x00, 0x04, 0x02, 0xA0,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x4E, 0x00, 0x10,
            0x60, 0x65,
            0x13, 0xA0,
            0xDE, 0xAD, 0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0xC0,
            0x56, 0x78, 0xAB, 0xCD, 0xAB, 0xCE, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x01, 0x80,
            0x01, 0x02, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x03, 0x00, 0x00, 0x00, 0xFF, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x56, 0x78,
            0xAB, 0xCD, 0xAB, 0xDE, 0x00, 0x00, 0x0A, 0x00,
            0x0C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let expected = SmaInvIdentify {
            dst: SmaEndpoint::dummy(),
            src: SmaEndpoint {
                susy_id: 0x5678,
                serial: 0xABCDABCE,
            },
            error_code: 0,
            counters: SmaInvCounter {
                packet_id: 1,
                ..Default::default()
            },
            identity: Some([
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03,
                0x00, 0x00, 0x00, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x01, 0x00, 0x56, 0x78, 0xAB, 0xCD, 0xAB, 0xDE, 0x00, 0x00,
                0x0A, 0x00, 0x0C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00,
            ]),
        };

        let mut cursor = Cursor::new(&serialized[..]);
        match SmaInvIdentify::deserialize(&mut cursor) {
            Err(e) => panic!("SmaInvIdentify deserialization failed: {e:?}"),
            Ok(cmd) => {
                assert_eq!(expected, cmd);
                assert_eq!(SmaInvIdentify::LENGTH_MAX, cursor.position());
            }
        }
    }
}
