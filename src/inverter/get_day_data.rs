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
    SmaInvHeader, SmaInvMeterValue, SmaPacketFooter, SmaPacketHeader, SmaSerde,
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
#[cfg(not(feature = "std"))]
use heapless::Vec;

/// A logical GetDayData message resquest/response.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SmaInvGetDayData {
    /// Destination application/device address.
    pub dst: SmaEndpoint,
    /// Source application/device address.
    pub src: SmaEndpoint,
    /// Non-zero in case of errors.
    pub error_code: u16,
    /// Packet counters.
    pub counters: SmaInvCounter,
    /// Start timestamp (request) or start record number (response).
    pub start_time_idx: u32,
    /// End timestamp (request) or end record number (response).
    pub end_time_idx: u32,
    #[cfg(not(feature = "std"))]
    /// Timestamped total energy production values.
    pub records: Vec<SmaInvMeterValue, { Self::MAX_RECORD_COUNT }>,
    /// Timestamped total energy production values.
    #[cfg(feature = "std")]
    pub records: Vec<SmaInvMeterValue>,
}

impl SmaInvGetDayData {
    pub const OPCODE: u32 = 0x020070;
    pub const LENGTH_MIN: usize = SmaPacketHeader::LENGTH
        + SmaInvHeader::LENGTH
        + 8
        + SmaPacketFooter::LENGTH;
    pub const LENGTH_MAX: usize =
        Self::LENGTH_MIN + Self::MAX_RECORD_COUNT * SmaInvMeterValue::LENGTH;
    pub const MAX_RECORD_COUNT: usize = 81;

    pub fn serialized_len(&self) -> usize {
        Self::LENGTH_MIN + self.records.len() * SmaInvMeterValue::LENGTH
    }
}

impl SmaSerde for SmaInvGetDayData {
    fn serialize(&self, buffer: &mut Cursor<&mut [u8]>) -> Result<()> {
        if self.records.len() > Self::MAX_RECORD_COUNT {
            return Err(Error::PayloadTooLarge {
                len: self.records.len(),
            });
        }

        let len = self.serialized_len();
        buffer.check_remaining(len)?;

        let data_len = len - SmaPacketHeader::LENGTH - SmaPacketFooter::LENGTH;
        let header = SmaPacketHeader {
            data_len,
            protocol: SmaPacketHeader::SMA_PROTOCOL_INV,
        };

        let (channel, dst_ctrl) = if self.records.is_empty() {
            (0, 0x00)
        } else {
            (1, 0xA0)
        };

        let inv_header = SmaInvHeader {
            wordcount: (data_len / 4) as u8,
            class: 0xE0,
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

        buffer.write_u32::<LittleEndian>(self.start_time_idx);
        buffer.write_u32::<LittleEndian>(self.end_time_idx);

        for record in &self.records {
            record.serialize(buffer)?;
        }

        SmaPacketFooter::default().serialize(buffer)?;

        Ok(())
    }

    fn deserialize(buffer: &mut Cursor<&[u8]>) -> Result<Self> {
        buffer.check_remaining(Self::LENGTH_MIN)?;

        let header = SmaPacketHeader::deserialize(buffer)?;
        header.check_protocol(SmaPacketHeader::SMA_PROTOCOL_INV)?;
        buffer.check_remaining(header.data_len)?;
        let padding_len = buffer.remaining() - header.data_len;

        let inv_header = SmaInvHeader::deserialize(buffer)?;
        inv_header.check_wordcount(header.data_len)?;
        inv_header.check_class(0xE0)?;
        inv_header.check_opcode(Self::OPCODE)?;

        let start_time_idx = buffer.read_u32::<LittleEndian>();
        let end_time_idx = buffer.read_u32::<LittleEndian>();

        let mut records = Vec::default();
        while buffer.remaining() - padding_len >= SmaInvMeterValue::LENGTH {
            let record = SmaInvMeterValue::deserialize(buffer)?;

            #[cfg(feature = "std")]
            records.push(record);
            #[cfg(not(feature = "std"))]
            if records.push(record).is_err() {
                return Err(Error::PayloadTooLarge {
                    len: records.len() + 1,
                });
            }
        }

        SmaPacketFooter::deserialize(buffer)?;

        Ok(Self {
            dst: inv_header.dst,
            src: inv_header.src,
            error_code: inv_header.error_code,
            counters: inv_header.counters,
            start_time_idx,
            end_time_idx,
            records,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_inv_get_day_data_serialization() {
        let message = SmaInvGetDayData {
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

        let mut buffer = [0u8; SmaInvGetDayData::LENGTH_MIN];
        let mut cursor = Cursor::new(&mut buffer[..]);

        if let Err(e) = message.serialize(&mut cursor) {
            panic!("SmaInvGetDayData serialization failed: {e:?}");
        }

        #[rustfmt::skip]
        let expected = [
            0x53, 0x4D, 0x41, 0x00, 0x00, 0x04, 0x02, 0xA0,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x26, 0x00, 0x10,
            0x60, 0x65,
            0x09, 0xE0,
            0x56, 0x78, 0xAB, 0xCD, 0xAB, 0xCE, 0x00, 0x00,
            0xDE, 0xAD, 0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x03, 0x80,
            0x00, 0x02, 0x00, 0x70,
            0x00, 0xF1, 0x53, 0x65, 0x80, 0xE1, 0x4E, 0x68,
            0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(SmaInvGetDayData::LENGTH_MIN, cursor.position());
        assert_eq!(expected, buffer);
    }

    #[test]
    fn test_sma_inv_get_day_data_deserialization() {
        #[rustfmt::skip]
        let serialized = [
            0x53, 0x4D, 0x41, 0x00, 0x00, 0x04, 0x02, 0xA0,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x26, 0x00, 0x10,
            0x60, 0x65,
            0x09, 0xE0,
            0x56, 0x78, 0xAB, 0xCD, 0xAB, 0xCE, 0x00, 0x00,
            0xDE, 0xAD, 0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x03, 0x80,
            0x00, 0x02, 0x00, 0x70,
            0x00, 0xF1, 0x53, 0x65, 0x80, 0xE1, 0x4E, 0x68,
            0x00, 0x00, 0x00, 0x00,
        ];

        let expected = SmaInvGetDayData {
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

        let mut cursor = Cursor::new(&serialized[..]);
        match SmaInvGetDayData::deserialize(&mut cursor) {
            Err(e) => panic!("SmaGetDayData deserialization failed: {e:?}"),
            Ok(message) => {
                assert_eq!(expected, message);
                assert_eq!(SmaInvGetDayData::LENGTH_MIN, cursor.position());
            }
        }
    }

    #[test]
    fn test_sma_inv_get_day_data_response_deserialization() {
        #[rustfmt::skip]
        let serialized = [
            0x53, 0x4D, 0x41, 0x00, 0x00, 0x04, 0x02, 0xA0,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x56, 0x00, 0x10,
            0x60, 0x65,
            0x15, 0xE0,
            0xDE, 0xAD, 0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0xA0,
            0x56, 0x78, 0xAB, 0xCD, 0xAB, 0xCE, 0x00, 0x00,
            0x00, 0x00, 0x03, 0x00, 0x08, 0x80,
            0x01, 0x02, 0x00, 0x70,
            0x04, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00,
            0x00, 0xF1, 0x53, 0x65, 0xF6, 0x97, 0xC2, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x2C, 0xF2, 0x53, 0x65, 0xFF, 0x97, 0xC2, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x58, 0xF3, 0x53, 0x65, 0x08, 0x98, 0xC2, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x84, 0xF4, 0x53, 0x65, 0x10, 0x98, 0xC2, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let expected = SmaInvGetDayData {
            dst: SmaEndpoint::dummy(),
            src: SmaEndpoint {
                susy_id: 0x5678,
                serial: 0xABCDABCE,
            },
            error_code: 0,
            counters: SmaInvCounter {
                packet_id: 8,
                fragment_id: 3,
                first_fragment: true,
            },
            start_time_idx: 4,
            end_time_idx: 8,
            records: {
                let mut records = Vec::default();
                #[allow(clippy::let_unit_value)]
                let _ = records.push(SmaInvMeterValue {
                    timestamp: 1700000000,
                    energy_wh: 12752886,
                });
                #[allow(clippy::let_unit_value)]
                let _ = records.push(SmaInvMeterValue {
                    timestamp: 1700000300,
                    energy_wh: 12752895,
                });
                #[allow(clippy::let_unit_value)]
                let _ = records.push(SmaInvMeterValue {
                    timestamp: 1700000600,
                    energy_wh: 12752904,
                });
                #[allow(clippy::let_unit_value)]
                let _ = records.push(SmaInvMeterValue {
                    timestamp: 1700000900,
                    energy_wh: 12752912,
                });
                records
            },
        };

        let mut cursor = Cursor::new(&serialized[..]);
        match SmaInvGetDayData::deserialize(&mut cursor) {
            Err(e) => panic!("SmaCmdGetDayData deserialization failed: {e:?}"),
            Ok(message) => {
                assert_eq!(expected, message);
                assert_eq!(
                    SmaInvGetDayData::LENGTH_MIN + 48,
                    cursor.position()
                );
            }
        }
    }
}
