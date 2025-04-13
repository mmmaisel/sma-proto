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

//! Common SMA packet serialization and deserialization structures and traits.

#[cfg(not(feature = "std"))]
use core::{
    clone::Clone,
    cmp::{Eq, PartialEq},
    fmt::Debug,
    prelude::rust_2021::derive,
    result::Result::{Err, Ok},
};

use byteorder_cursor::{BigEndian, Cursor};

use super::{Error, Result};

/// Interface for (de)serialization of SMA speedwire messages.
pub trait SmaSerde {
    /// Serialize given object into buffer.
    fn serialize(&self, buffer: &mut Cursor<&mut [u8]>) -> Result<()>;
    /// Deserialize buffer into object.
    /// The supplied slice must contain exactly one packet.
    fn deserialize(buffer: &mut Cursor<&[u8]>) -> Result<Self>
    where
        Self: Sized;
}

/// Common SMA speedwire packet header.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct SmaPacketHeader {
    /// Length of the following data payload.
    pub data_len: usize,
    /// Sub-protocol type ID.
    pub protocol: u16,
}

impl SmaPacketHeader {
    /// Serialized length of the common packet header.
    pub const LENGTH: usize = 18;
    pub const SMA_FOURCC: u32 = 0x534D4100; // SMA\0
    const START_TAG_LEN: usize = 4;
    const START_TAG: u16 = 0x02A0;
    const DEFAULT_GROUP: u32 = 1;
    /// SMA inverter sub-protocol ID.
    pub const SMA_PROTOCOL_INV: u16 = 0x6065;
    /// SMA energymeter sub-protocol ID.
    pub const SMA_PROTOCOL_EM: u16 = 0x6069;
    const SMA_VERSION: u16 = 0x10;

    pub fn check_protocol(&self, protocol: u16) -> Result<()> {
        if self.protocol != protocol {
            return Err(Error::UnsupportedProtocol {
                protocol: self.protocol,
            });
        }

        Ok(())
    }
}

impl SmaSerde for SmaPacketHeader {
    fn serialize(&self, buffer: &mut Cursor<&mut [u8]>) -> Result<()> {
        buffer.check_remaining(Self::LENGTH)?;

        buffer.write_u32::<BigEndian>(Self::SMA_FOURCC);
        // Length of the header in 32bit words without the protocol field.
        buffer.write_u16::<BigEndian>((Self::LENGTH / 4) as u16);
        // Constant start tag value.
        buffer.write_u16::<BigEndian>(Self::START_TAG);
        // Default group ID.
        buffer.write_u32::<BigEndian>(Self::DEFAULT_GROUP);
        buffer.write_u16::<BigEndian>((self.data_len + 2) as u16);
        // SMA speedwire version.
        buffer.write_u16::<BigEndian>(Self::SMA_VERSION);
        buffer.write_u16::<BigEndian>(self.protocol);

        Ok(())
    }

    fn deserialize(buffer: &mut Cursor<&[u8]>) -> Result<Self> {
        buffer.check_remaining(Self::LENGTH)?;

        let fourcc = buffer.read_u32::<BigEndian>();
        if fourcc != Self::SMA_FOURCC {
            return Err(Error::InvalidFourCC { fourcc });
        }

        let len = buffer.read_u16::<BigEndian>();
        if len != (Self::START_TAG_LEN) as u16 {
            return Err(Error::InvalidStartTagLen { len });
        }

        let tag = buffer.read_u16::<BigEndian>();
        if tag != Self::START_TAG {
            return Err(Error::InvalidStartTag { tag });
        }

        let group = buffer.read_u32::<BigEndian>();
        if group != Self::DEFAULT_GROUP {
            return Err(Error::InvalidGroup { group });
        }

        let data_len = (buffer.read_u16::<BigEndian>() - 2) as usize;

        let version = buffer.read_u16::<BigEndian>();
        if version != Self::SMA_VERSION {
            return Err(Error::UnsupportedVersion { version });
        }

        let protocol = buffer.read_u16::<BigEndian>();

        Ok(Self { data_len, protocol })
    }
}

/// Footer with optional variable length zero padding at the and of an
/// SMA packet.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub(crate) struct SmaPacketFooter {}

impl SmaPacketFooter {
    /// Serialized length of a short SMA speedwire packet footer.
    pub const LENGTH_SHORT: usize = 2;
    /// Serialized length of a normal SMA speedwire packet footer.
    pub const LENGTH: usize = 4;
}

impl SmaSerde for SmaPacketFooter {
    fn serialize(&self, buffer: &mut Cursor<&mut [u8]>) -> Result<()> {
        buffer.check_remaining(Self::LENGTH)?;
        buffer.write_u32::<BigEndian>(0);

        Ok(())
    }

    fn deserialize(buffer: &mut Cursor<&[u8]>) -> Result<Self> {
        buffer.check_remaining(Self::LENGTH_SHORT)?;

        while buffer.remaining() >= Self::LENGTH {
            let padding = buffer.read_u32::<BigEndian>();
            if padding != 0 {
                return Err(Error::InvalidPadding { padding });
            }
        }

        if buffer.remaining() == Self::LENGTH_SHORT {
            let padding = buffer.read_u16::<BigEndian>() as u32;
            if padding != 0 {
                return Err(Error::InvalidPadding { padding });
            }
        }

        let trailing = buffer.remaining();
        if trailing != 0 {
            return Err(Error::BufferNotConsumed { trailing });
        }

        Ok(Self {})
    }
}

/// Identifies a SMA speedwire communication endpoint.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SmaEndpoint {
    /// SMA Update System-ID.
    pub susy_id: u16,
    /// Device serial number.
    pub serial: u32,
}

impl SmaEndpoint {
    const LENGTH: usize = 6;

    /// The libraries dummy SUSy ID and serial SMA endpoint.
    pub fn dummy() -> Self {
        Self {
            susy_id: 0xDEAD,
            serial: 0xDEADBEEF,
        }
    }

    /// Broadcast SUSy ID and serial SMA endpoint.
    pub fn broadcast() -> Self {
        Self {
            susy_id: 0xFFFF,
            serial: 0xFFFFFFFF,
        }
    }
}

impl SmaSerde for SmaEndpoint {
    fn serialize(&self, buffer: &mut Cursor<&mut [u8]>) -> Result<()> {
        buffer.check_remaining(Self::LENGTH)?;
        buffer.write_u16::<BigEndian>(self.susy_id);
        buffer.write_u32::<BigEndian>(self.serial);

        Ok(())
    }

    fn deserialize(buffer: &mut Cursor<&[u8]>) -> Result<Self> {
        buffer.check_remaining(Self::LENGTH)?;

        Ok(Self {
            susy_id: buffer.read_u16::<BigEndian>(),
            serial: buffer.read_u32::<BigEndian>(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_packet_header_serialization() {
        let header = SmaPacketHeader {
            data_len: 8,
            protocol: SmaPacketHeader::SMA_PROTOCOL_EM,
        };
        let mut buffer = [0u8; SmaPacketHeader::LENGTH];
        let mut cursor = Cursor::new(&mut buffer[..]);

        if let Err(e) = header.serialize(&mut cursor) {
            panic!("SmaPacketHeader serialization failed: {e:?}");
        }

        #[rustfmt::skip]
        let expected = [
            0x53, 0x4D, 0x41, 0x00,
            0x00, 0x04,
            0x02, 0xA0,
            0x00, 0x00, 0x00, 0x01,
            0x00, 0x0A,
            0x00, 0x10,
            0x60, 0x69,
        ];
        assert_eq!(SmaPacketHeader::LENGTH, cursor.position());
        assert_eq!(expected, buffer);
    }

    #[test]
    fn test_sma_packet_header_deserialization() {
        #[rustfmt::skip]
        let serialized = [
            0x53, 0x4D, 0x41, 0x00,
            0x00, 0x04,
            0x02, 0xA0,
            0x00, 0x00, 0x00, 0x01,
            0x00, 0x0A,
            0x00, 0x10,
            0x60, 0x69,
        ];

        let expected = SmaPacketHeader {
            data_len: 8,
            protocol: SmaPacketHeader::SMA_PROTOCOL_EM,
        };

        let mut cursor = Cursor::new(&serialized[..]);
        match SmaPacketHeader::deserialize(&mut cursor) {
            Err(e) => panic!("SmaPacketHeader deserialization failed: {e:?}"),
            Ok(header) => {
                assert_eq!(expected, header);
                assert_eq!(SmaPacketHeader::LENGTH, cursor.position());
            }
        }
    }

    #[test]
    fn test_sma_packet_footer_serialization() {
        let token = SmaPacketFooter::default();
        let mut buffer = [1u8; 4];
        let mut cursor = Cursor::new(&mut buffer[..]);

        if let Err(e) = token.serialize(&mut cursor) {
            panic!("SmaPacketFooter serialization failed: {e:?}");
        }

        assert_eq!(4, cursor.position());
        assert_eq!([0u8; 4], buffer);
    }

    #[test]
    fn test_sma_packet_footer_deserialization() {
        let buffer = [0u8; 4];
        let mut cursor = Cursor::new(&buffer[..]);

        if let Err(e) = SmaPacketFooter::deserialize(&mut cursor) {
            panic!("SmaPacketFooter deserialization failed: {e:?}");
        }
        assert_eq!(4, cursor.position());

        let buffer = [0u8; 2];
        let mut cursor = Cursor::new(&buffer[..]);

        if let Err(e) = SmaPacketFooter::deserialize(&mut cursor) {
            panic!("SmaPacketFooter deserialization failed: {e:?}");
        }
        assert_eq!(2, cursor.position());

        let buffer = [0u8; 12];
        let mut cursor = Cursor::new(&buffer[..]);

        if let Err(e) = SmaPacketFooter::deserialize(&mut cursor) {
            panic!("SmaPacketFooter deserialization failed: {e:?}");
        }
        assert_eq!(12, cursor.position());
    }

    #[test]
    fn test_sma_endpoint_serialization() {
        let endpoint = SmaEndpoint {
            susy_id: 0x1234,
            serial: 0xDEADBEEF,
        };
        let mut buffer = [0u8; SmaEndpoint::LENGTH];
        let mut cursor = Cursor::new(&mut buffer[..]);

        if let Err(e) = endpoint.serialize(&mut cursor) {
            panic!("SmaEndpoint serialization failed: {e:?}");
        }

        #[rustfmt::skip]
        let expected = [
            0x12, 0x34,
            0xDE, 0xAD, 0xBE, 0xEF,
        ];
        assert_eq!(SmaEndpoint::LENGTH, cursor.position());
        assert_eq!(expected, buffer);
    }

    #[test]
    fn test_sma_endpoint_deserialization() {
        #[rustfmt::skip]
        let serialized = [
            0x12, 0x34,
            0xDE, 0xAD, 0xBE, 0xEF,
        ];

        let expected = SmaEndpoint {
            susy_id: 0x1234,
            serial: 0xDEADBEEF,
        };

        let mut cursor = Cursor::new(&serialized[..]);
        match SmaEndpoint::deserialize(&mut cursor) {
            Err(e) => panic!("SmaEndpoint deserialization failed: {e:?}"),
            Ok(endpoint) => {
                assert_eq!(expected, endpoint);
                assert_eq!(SmaEndpoint::LENGTH, cursor.position());
            }
        };
    }
}
