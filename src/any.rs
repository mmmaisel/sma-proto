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
    result::Result::{Err, Ok},
};

use byteorder_cursor::{BigEndian, Cursor};

use super::{
    energymeter::{ObisValue, SmaEmMessageBase},
    inverter::{
        SmaInvGetDayDataBase, SmaInvHeader, SmaInvIdentify, SmaInvLogin,
        SmaInvLogout, SmaInvMeterValue,
    },
    packet::SmaPacketHeader,
    Error, Result, SmaContainer, SmaSerde,
};

/// Container that can hold any supported SMA speedwire message.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AnySmaMessageBase<
    V: SmaContainer<ObisValue>,
    W: SmaContainer<SmaInvMeterValue>,
> {
    EmMessage(SmaEmMessageBase<V>),
    InvGetDayData(SmaInvGetDayDataBase<W>),
    InvIdentify(SmaInvIdentify),
    InvLogin(SmaInvLogin),
    InvLogout(SmaInvLogout),
}

impl<V: SmaContainer<ObisValue>, W: SmaContainer<SmaInvMeterValue>> SmaSerde
    for AnySmaMessageBase<V, W>
{
    fn serialize(&self, buffer: &mut Cursor<&mut [u8]>) -> Result<()> {
        match self {
            Self::EmMessage(x) => x.serialize(buffer),
            Self::InvGetDayData(x) => x.serialize(buffer),
            Self::InvIdentify(x) => x.serialize(buffer),
            Self::InvLogin(x) => x.serialize(buffer),
            Self::InvLogout(x) => x.serialize(buffer),
        }
    }

    fn deserialize(buffer: &mut Cursor<&[u8]>) -> Result<Self> {
        buffer.check_remaining(SmaPacketHeader::LENGTH)?;

        let fourcc = buffer.peek_u32::<BigEndian>(0);
        if fourcc != SmaPacketHeader::SMA_FOURCC {
            return Err(Error::InvalidFourCC { fourcc });
        }

        let protocol = buffer.peek_u16::<BigEndian>(16);
        let message = match protocol {
            SmaPacketHeader::SMA_PROTOCOL_EM => {
                Self::EmMessage(SmaEmMessageBase::deserialize(buffer)?)
            }
            SmaPacketHeader::SMA_PROTOCOL_INV => {
                buffer.check_remaining(
                    SmaPacketHeader::LENGTH + SmaInvHeader::LENGTH,
                )?;
                let opcode = buffer.peek_u24::<BigEndian>(43);
                match opcode {
                    SmaInvGetDayDataBase::<W>::OPCODE => Self::InvGetDayData(
                        SmaInvGetDayDataBase::deserialize(buffer)?,
                    ),
                    SmaInvIdentify::OPCODE => {
                        Self::InvIdentify(SmaInvIdentify::deserialize(buffer)?)
                    }
                    SmaInvLogin::OPCODE => {
                        Self::InvLogin(SmaInvLogin::deserialize(buffer)?)
                    }
                    SmaInvLogout::OPCODE => {
                        Self::InvLogout(SmaInvLogout::deserialize(buffer)?)
                    }
                    opcode => return Err(Error::UnsupportedOpcode { opcode }),
                }
            }
            protocol => return Err(Error::UnsupportedProtocol { protocol }),
        };

        Ok(message)
    }
}

#[cfg(feature = "std")]
/// An [AnySmaMessageBase] using std [Vec] as storage.
pub type AnySmaMessageStd =
    AnySmaMessageBase<Vec<ObisValue>, Vec<SmaInvMeterValue>>;
#[cfg(feature = "heapless")]
/// An [AnySmaMessageBase] using [heapless::Vec] as storage.
pub type AnySmaMessageHeapless = AnySmaMessageBase<
    heapless::Vec<ObisValue, { SmaEmMessageBase::<()>::MAX_RECORD_COUNT }>,
    heapless::Vec<
        SmaInvMeterValue,
        { SmaInvGetDayDataBase::<()>::MAX_RECORD_COUNT },
    >,
>;

#[cfg(feature = "std")]
/// An [AnySmaMessageBase] using default storage based on selected features.
pub type AnySmaMessage = AnySmaMessageStd;
#[cfg(not(feature = "std"))]
/// An [AnySmaMessageBase] using default storage based on selected features.
pub type AnySmaMessage = AnySmaMessageHeapless;

#[cfg(test)]
mod tests {
    use heapless::Vec;

    use super::*;
    use crate::{
        energymeter::{ObisValue, SmaEmMessageHeapless},
        inverter::{SmaInvCounter, SmaInvGetDayDataHeapless},
        packet::SmaEndpoint,
    };

    #[test]
    fn test_any_em_message_deserialization() {
        #[rustfmt::skip]
        let serialized = [
            0x53, 0x4D, 0x41, 0x00, 0x00, 0x04, 0x02, 0xA0,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x14, 0x00, 0x10,
            0x60, 0x69,
            0xDE, 0xAD,
            0x11, 0x22, 0x33, 0x44,
            0xAA, 0xBB, 0xCC, 0xDD,
            0x00, 0x01, 0x04, 0x00, 0x01, 0x02, 0x03, 0x04,
            0x00, 0x00, 0x00, 0x00,
        ];

        let expected = AnySmaMessageHeapless::EmMessage(SmaEmMessageHeapless {
            src: SmaEndpoint {
                susy_id: 0xDEAD,
                serial: 0x11223344,
            },
            timestamp_ms: 0xAABBCCDD,
            payload: {
                let mut message = Vec::default();
                let _ = message.push(ObisValue {
                    id: 0x010400,
                    value: 0x01020304,
                });
                message
            },
        });

        let mut cursor = Cursor::new(&serialized[..]);
        match AnySmaMessageHeapless::deserialize(&mut cursor) {
            Err(e) => panic!("AnySmaMessage deserialization failed: {e:?}"),
            Ok(message) => {
                assert_eq!(expected, message);
                assert_eq!(40, cursor.position());
            }
        }
    }

    #[test]
    fn test_any_inv_login_response_deserialization() {
        #[rustfmt::skip]
        let serialized = [
            0x53, 0x4D, 0x41, 0x00, 0x00, 0x04, 0x02, 0xA0,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x2E, 0x00, 0x10,
            0x60, 0x65,
            0x0B, 0xE0,
            0xDE, 0xAD, 0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x01,
            0x56, 0x78, 0xAB, 0xCD, 0xAB, 0xCE, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x02, 0x80,
            0x0D, 0x04, 0xFD, 0xFF,
            0x07, 0x00, 0x00, 0x00, 0x84, 0x03, 0x00, 0x00,
            0x00, 0xF1, 0x53, 0x65, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let expected = AnySmaMessage::InvLogin(SmaInvLogin {
            dst: SmaEndpoint::dummy(),
            src: SmaEndpoint {
                susy_id: 0x5678,
                serial: 0xABCDABCE,
            },
            counters: SmaInvCounter {
                packet_id: 2,
                ..Default::default()
            },
            timestamp: 1700000000,
            password: None,
            ..Default::default()
        });

        let mut cursor = Cursor::new(&serialized[..]);
        match AnySmaMessage::deserialize(&mut cursor) {
            Err(e) => panic!("AnySmaMessage deserialization failed: {e:?}"),
            Ok(message) => {
                assert_eq!(expected, message);
                assert_eq!(SmaInvLogin::LENGTH_MIN, cursor.position());
            }
        }
    }

    #[test]
    fn test_any_inv_logout_serialization() {
        let cmd = AnySmaMessage::InvLogout(SmaInvLogout {
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
        });

        let mut buffer = [0u8; SmaInvLogout::LENGTH];
        let mut cursor = Cursor::new(&mut buffer[..]);

        if let Err(e) = cmd.serialize(&mut cursor) {
            panic!("AnySmaMessage serialization failed: {e:?}");
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
    fn reject_random_junk() {
        let serialized = [
            0xCB, 0xF2, 0x87, 0x99, 0xA7, 0x35, 0x70, 0x5E, 0xAE, 0x51, 0x2B,
            0xEE, 0xC9, 0x66, 0x08, 0xF2, 0x7F, 0x84, 0x54, 0x72, 0xC5, 0x23,
            0x77, 0x2B, 0xF1, 0x01, 0x3F, 0x27, 0xDC, 0x2F, 0x26, 0x05, 0xE8,
            0xCC, 0xC4, 0xAC, 0x38, 0x24, 0x47, 0xBD, 0x27, 0x28, 0xEB, 0x8A,
            0x4A, 0x93, 0x97, 0x22, 0xBC, 0x69, 0x68, 0x92, 0x07, 0x5D, 0xE4,
            0xE8, 0x1D, 0x2D, 0xE0, 0x2D, 0xB3, 0x8C, 0x22, 0x19,
        ];

        let mut cursor = Cursor::new(&serialized[..]);
        if let Ok(x) = AnySmaMessage::deserialize(&mut cursor) {
            panic!("Deserialized junk as {x:?}");
        }
    }

    #[test]
    fn serialize_into_too_small_buffer() {
        let message = SmaInvGetDayDataHeapless {
            src: SmaEndpoint::dummy(),
            dst: SmaEndpoint {
                susy_id: 0x5678,
                serial: 0xABCDABCE,
            },
            error_code: 0,
            counters: SmaInvCounter {
                packet_id: 3,
                ..Default::default()
            },
            start_time_idx: 1700000000,
            end_time_idx: 1750000000,
            records: Vec::new(),
        };

        let mut buffer = [0u8; SmaInvGetDayDataHeapless::LENGTH_MIN - 1];
        let mut cursor = Cursor::new(&mut buffer[..]);

        if let Ok(x) = message.serialize(&mut cursor) {
            panic!("Serialized message into too small buffer {x:?}");
        }
    }
}
