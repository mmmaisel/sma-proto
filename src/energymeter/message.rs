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
use byteorder_cursor::Cursor;

use super::{
    Error, ObisValue, Result, SmaEmHeader, SmaEndpoint, SmaPacketFooter,
    SmaPacketHeader, SmaSerde,
};
use crate::SmaContainer;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
/// A logical SMA energymeter message.
pub struct SmaEmMessageBase<V> {
    /// Source endpoint address.
    pub src: SmaEndpoint,
    /// Overflowing timestamp in milliseconds.
    pub timestamp_ms: u32,
    /// Vector of OBIS data.
    pub payload: V,
}

impl<V> SmaEmMessageBase<V> {
    /// Minimum serialized length of the energymeter message.
    pub const LENGTH_MIN: usize =
        SmaPacketHeader::LENGTH + SmaEmHeader::LENGTH + SmaPacketFooter::LENGTH;
    /// Maximum serialized length of the energymeter message.
    pub const LENGTH_MAX: usize =
        Self::LENGTH_MIN + Self::MAX_RECORD_COUNT * ObisValue::LENGTH_MAX;
    /// Maximum number of OBIS values in the payload.
    pub const MAX_RECORD_COUNT: usize = 80;
}

impl<V: SmaContainer<ObisValue>> SmaEmMessageBase<V> {
    /// Returns total serialized message length.
    pub fn serialized_len(&self) -> usize {
        Self::LENGTH_MIN
            + self
                .payload
                .iter()
                .map(ObisValue::serialized_len)
                .sum::<usize>()
    }
}

impl<V: SmaContainer<ObisValue>> SmaSerde for SmaEmMessageBase<V> {
    fn serialize(&self, buffer: &mut Cursor<&mut [u8]>) -> Result<()> {
        if self.payload.len() > Self::MAX_RECORD_COUNT {
            return Err(Error::PayloadTooLarge {
                len: self.payload.len(),
            });
        }

        let len = self.serialized_len();
        buffer.check_remaining(len)?;

        let header = SmaPacketHeader {
            data_len: len - SmaPacketHeader::LENGTH - SmaPacketFooter::LENGTH,
            protocol: SmaPacketHeader::SMA_PROTOCOL_EM,
        };

        let em_header = SmaEmHeader {
            src: self.src.clone(),
            timestamp_ms: self.timestamp_ms,
        };

        header.serialize(buffer)?;
        em_header.serialize(buffer)?;

        for obis in self.payload.iter() {
            obis.validate()?;
            obis.serialize(buffer)?;
        }

        SmaPacketFooter::default().serialize(buffer)?;

        Ok(())
    }

    fn deserialize(buffer: &mut Cursor<&[u8]>) -> Result<Self> {
        buffer.check_remaining(Self::LENGTH_MIN)?;

        let header = SmaPacketHeader::deserialize(buffer)?;
        header.check_protocol(SmaPacketHeader::SMA_PROTOCOL_EM)?;
        buffer.check_remaining(header.data_len)?;
        let padding_len = buffer.remaining() - header.data_len;

        let em_header = SmaEmHeader::deserialize(buffer)?;

        let mut payload = V::default();
        while buffer.remaining() - padding_len >= ObisValue::LENGTH_MIN {
            let obis = ObisValue::deserialize(buffer)?;
            obis.validate()?;

            if payload.push(obis).is_err() {
                return Err(Error::PayloadTooLarge {
                    len: payload.len() + 1,
                });
            }
        }

        SmaPacketFooter::deserialize(buffer)?;

        let message = Self {
            src: em_header.src,
            timestamp_ms: em_header.timestamp_ms,
            payload,
        };

        Ok(message)
    }
}

#[cfg(feature = "std")]
/// An [SmaEmMessageBase] using std [Vec] as storage.
pub type SmaEmMessageStd = SmaEmMessageBase<Vec<ObisValue>>;
#[cfg(feature = "heapless")]
/// An [SmaEmMessageBase] using [heapless::Vec] as storage.
pub type SmaEmMessageHeapless = SmaEmMessageBase<
    heapless::Vec<ObisValue, { SmaEmMessageBase::<()>::MAX_RECORD_COUNT }>,
>;

#[cfg(feature = "std")]
/// An [SmaEmMessageBase] using default storage based on selected features.
pub type SmaEmMessage = SmaEmMessageStd;
#[cfg(not(feature = "std"))]
/// An [SmaEmMessageBase] using default storage based on selected features.
pub type SmaEmMessage = SmaEmMessageHeapless;

#[cfg(test)]
mod tests {
    use heapless::Vec;

    use super::*;

    #[test]
    fn test_sma_em_message_serialization() {
        let message = SmaEmMessageHeapless {
            src: SmaEndpoint::dummy(),
            timestamp_ms: 0xAABBCCDD,
            payload: {
                let mut message = Vec::default();
                let _ = message.push(ObisValue {
                    id: 0x010400,
                    value: 0x01020304,
                });
                let _ = message.push(ObisValue {
                    id: 0x010800,
                    value: 0x1020304050607080,
                });
                let _ = message.push(ObisValue {
                    id: 0x90000000,
                    value: 0x02001252,
                });
                message
            },
        };

        let mut buffer = [0u8; 60];
        let mut cursor = Cursor::new(&mut buffer[..]);

        if let Err(e) = message.serialize(&mut cursor) {
            panic!("SmaEmMessage serialization failed: {e:?}");
        }

        #[rustfmt::skip]
        let expected = [
            0x53, 0x4D, 0x41, 0x00, 0x00, 0x04, 0x02, 0xA0,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x28, 0x00, 0x10,
            0x60, 0x69,
            0xDE, 0xAD,
            0xDE, 0xAD, 0xBE, 0xEF,
            0xAA, 0xBB, 0xCC, 0xDD,
            0x00, 0x01, 0x04, 0x00, 0x01, 0x02, 0x03, 0x04,
            0x00, 0x01, 0x08, 0x00, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80,
            0x90, 0x00, 0x00, 0x00, 0x02, 0x00, 0x12, 0x52,
            0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(60, cursor.position());
        assert_eq!(expected, buffer);
    }

    #[test]
    fn test_sma_em_message_deserialization() {
        #[rustfmt::skip]
        let serialized = [
            0x53, 0x4D, 0x41, 0x00, 0x00, 0x04, 0x02, 0xA0,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x28, 0x00, 0x10,
            0x60, 0x69,
            0xDE, 0xAD,
            0xDE, 0xAD, 0xBE, 0xEF,
            0xAA, 0xBB, 0xCC, 0xDD,
            0x00, 0x01, 0x04, 0x00, 0x01, 0x02, 0x03, 0x04,
            0x00, 0x01, 0x08, 0x00, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80,
            0x90, 0x00, 0x00, 0x00, 0x02, 0x00, 0x12, 0x52,
            0x00, 0x00, 0x00, 0x00,
        ];

        let expected = SmaEmMessageHeapless {
            src: SmaEndpoint::dummy(),
            timestamp_ms: 0xAABBCCDD,
            payload: {
                let mut message = Vec::default();
                let _ = message.push(ObisValue {
                    id: 0x010400,
                    value: 0x01020304,
                });
                let _ = message.push(ObisValue {
                    id: 0x010800,
                    value: 0x1020304050607080,
                });
                let _ = message.push(ObisValue {
                    id: 0x90000000,
                    value: 0x02001252,
                });
                message
            },
        };

        let mut cursor = Cursor::new(&serialized[..]);
        match SmaEmMessageHeapless::deserialize(&mut cursor) {
            Err(e) => panic!("SmaEmMessage deserialization failed: {e:?}"),
            Ok(message) => {
                assert_eq!(expected, message);
                assert_eq!(60, cursor.position());
            }
        }
    }
}
