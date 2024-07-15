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

/// Invalid input password error.
#[derive(Clone, Debug)]
pub struct InvalidPasswordError();

/// A logical SMA inverter login message.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SmaInvLogin {
    /// Destination application/device address.
    pub dst: SmaEndpoint,
    /// Source application/device address.
    pub src: SmaEndpoint,
    /// Non-zero in case of errors.
    pub error_code: u16,
    /// Packet counters.
    pub counters: SmaInvCounter,
    /// User group ID on the inverter.
    pub user_group: u32,
    /// Session timeout in seconds.
    pub timeout: u32,
    /// Unix timestamp of the request.
    pub timestamp: u32,
    /// Up to 12 character zero padded password.
    /// Required for command, usually absent in response.
    pub password: Option<[u8; Self::PASSWORD_LEN]>,
}

impl Default for SmaInvLogin {
    fn default() -> Self {
        Self {
            dst: SmaEndpoint::default(),
            src: SmaEndpoint::default(),
            error_code: 0,
            counters: SmaInvCounter::default(),
            user_group: 7,
            timeout: 900,
            timestamp: 0,
            password: None,
        }
    }
}

impl SmaSerde for SmaInvLogin {
    fn serialize(&self, buffer: &mut Cursor<&mut [u8]>) -> Result<()> {
        let data_len = if self.password.is_some() {
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

        let (class, channel) = if self.password.is_some() {
            if self.error_code == 0 {
                (0xA0, 0x0C)
            } else {
                (0xD0, 0x0C)
            }
        } else {
            (0xE0, 0x0D)
        };

        let inv_header = SmaInvHeader {
            wordcount: (data_len / 4) as u8,
            class,
            dst: self.dst.clone(),
            dst_ctrl: 1,
            src: self.src.clone(),
            src_ctrl: 1,
            error_code: self.error_code,
            counters: self.counters.clone(),
            cmd: SmaCmdWord {
                channel,
                opcode: Self::OPCODE,
            },
        };

        header.serialize(buffer)?;
        inv_header.serialize(buffer)?;

        buffer.write_u32::<LittleEndian>(self.user_group);
        buffer.write_u32::<LittleEndian>(self.timeout);
        buffer.write_u32::<LittleEndian>(self.timestamp);
        buffer.write_u32::<LittleEndian>(0); // padding

        if let Some(password) = &self.password {
            for char in password {
                buffer.write_u8(char + 0x88);
            }
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
        if inv_header.check_class(0xA0).is_err()
            && inv_header.check_class(0xD0).is_err()
        {
            inv_header.check_class(0xE0)?;
        }
        inv_header.check_opcode(Self::OPCODE)?;

        let user_group = buffer.read_u32::<LittleEndian>();
        let timeout = buffer.read_u32::<LittleEndian>();
        let timestamp = buffer.read_u32::<LittleEndian>();
        let padding = buffer.read_u32::<LittleEndian>();
        if padding != 0 {
            return Err(Error::InvalidPadding { padding });
        }

        let payload_len = header.data_len - SmaInvHeader::LENGTH;
        let password = if payload_len >= Self::PAYLOAD_MAX {
            let mut password = [0; Self::PASSWORD_LEN];
            for char in password.iter_mut() {
                *char = buffer.read_u8() - 0x88;
            }
            Some(password)
        } else {
            None
        };

        SmaPacketFooter::deserialize(buffer)?;

        Ok(Self {
            dst: inv_header.dst,
            src: inv_header.src,
            error_code: inv_header.error_code,
            counters: inv_header.counters,
            user_group,
            timeout,
            timestamp,
            password,
        })
    }
}

impl SmaInvLogin {
    pub const OPCODE: u32 = 0x04FDFF;
    pub const LENGTH_MIN: usize = SmaPacketHeader::LENGTH
        + SmaInvHeader::LENGTH
        + Self::PAYLOAD_MIN
        + SmaPacketFooter::LENGTH;
    pub const LENGTH_MAX: usize = SmaPacketHeader::LENGTH
        + SmaInvHeader::LENGTH
        + Self::PAYLOAD_MAX
        + SmaPacketFooter::LENGTH;
    pub const PAYLOAD_MIN: usize = 16;
    pub const PAYLOAD_MAX: usize = 28;
    pub const PASSWORD_LEN: usize = 12;

    pub fn pw_from_str(
        passwd: &str,
    ) -> core::result::Result<[u8; Self::PASSWORD_LEN], InvalidPasswordError>
    {
        let mut buffer = [0; Self::PASSWORD_LEN];
        for (src, dst) in passwd.chars().zip(buffer.iter_mut()) {
            if !src.is_ascii() {
                return Err(InvalidPasswordError());
            }
            *dst = src as u8;
        }

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_inv_login_serialization() {
        let message = SmaInvLogin {
            src: SmaEndpoint::dummy(),
            dst: SmaEndpoint {
                susy_id: 0x5678,
                serial: 0xABCDABCE,
            },
            counters: SmaInvCounter {
                packet_id: 2,
                ..Default::default()
            },
            timestamp: 1700000000,
            password: Some(SmaInvLogin::pw_from_str("12345").unwrap()),
            ..Default::default()
        };

        let mut buffer = [0u8; SmaInvLogin::LENGTH_MAX];
        let mut cursor = Cursor::new(&mut buffer[..]);

        if let Err(e) = message.serialize(&mut cursor) {
            panic!("SmaInvLogin serialization failed: {e:?}");
        }

        #[rustfmt::skip]
        let expected = [
            0x53, 0x4D, 0x41, 0x00, 0x00, 0x04, 0x02, 0xA0,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x3A, 0x00, 0x10,
            0x60, 0x65,
            0x0E, 0xA0,
            0x56, 0x78, 0xAB, 0xCD, 0xAB, 0xCE, 0x00, 0x01,
            0xDE, 0xAD, 0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x02, 0x80,
            0x0C, 0x04, 0xFD, 0xFF,
            0x07, 0x00, 0x00, 0x00, 0x84, 0x03, 0x00, 0x00,
            0x00, 0xF1, 0x53, 0x65, 0x00, 0x00, 0x00, 0x00,
            0xB9, 0xBA, 0xBB, 0xBC, 0xBD, 0x88, 0x88, 0x88,
            0x88, 0x88, 0x88, 0x88,
            0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(SmaInvLogin::LENGTH_MAX, cursor.position());
        assert_eq!(expected, buffer);
    }

    #[test]
    fn test_sma_inv_login_deserialization() {
        #[rustfmt::skip]
        let serialized = [
            0x53, 0x4D, 0x41, 0x00, 0x00, 0x04, 0x02, 0xA0,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x3A, 0x00, 0x10,
            0x60, 0x65,
            0x0E, 0xA0,
            0x56, 0x78, 0xAB, 0xCD, 0xAB, 0xCE, 0x00, 0x01,
            0xDE, 0xAD, 0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x02, 0x80,
            0x0C, 0x04, 0xFD, 0xFF,
            0x07, 0x00, 0x00, 0x00, 0x84, 0x03, 0x00, 0x00,
            0x00, 0xF1, 0x53, 0x65, 0x00, 0x00, 0x00, 0x00,
            0xB9, 0xBA, 0xBB, 0xBC, 0xBD, 0x88, 0x88, 0x88,
            0x88, 0x88, 0x88, 0x88,
            0x00, 0x00, 0x00, 0x00,
        ];

        let expected = SmaInvLogin {
            src: SmaEndpoint::dummy(),
            dst: SmaEndpoint {
                susy_id: 0x5678,
                serial: 0xABCDABCE,
            },
            counters: SmaInvCounter {
                packet_id: 2,
                ..Default::default()
            },
            timestamp: 1700000000,
            password: Some(SmaInvLogin::pw_from_str("12345").unwrap()),
            ..Default::default()
        };

        let mut cursor = Cursor::new(&serialized[..]);
        match SmaInvLogin::deserialize(&mut cursor) {
            Err(e) => panic!("SmaInvLogin deserialization failed: {e:?}"),
            Ok(message) => {
                assert_eq!(expected, message);
                assert_eq!(SmaInvLogin::LENGTH_MAX, cursor.position());
            }
        }
    }

    #[test]
    fn test_sma_inv_login_response_deserialization() {
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

        let expected = SmaInvLogin {
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
        };

        let mut cursor = Cursor::new(&serialized[..]);
        match SmaInvLogin::deserialize(&mut cursor) {
            Err(e) => panic!("SmaInvLogin deserialization failed: {e:?}"),
            Ok(message) => {
                assert_eq!(expected, message);
                assert_eq!(SmaInvLogin::LENGTH_MIN, cursor.position());
            }
        }
    }

    #[test]
    fn test_sma_inv_login_failed_response_deserialization() {
        #[rustfmt::skip]
        let serialized = [
            0x53, 0x4D, 0x41, 0x00, 0x00, 0x04, 0x02, 0xA0,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x3A, 0x00, 0x10,
            0x60, 0x65,
            0x0E, 0xD0,
            0xDE, 0xAD, 0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x01,
            0x56, 0x78, 0xAB, 0xCD, 0xAB, 0xCE, 0x00, 0x01,
            0x00, 0x01, 0x00, 0x00, 0x02, 0x80,
            0x0D, 0x04, 0xFD, 0xFF,
            0x07, 0x00, 0x00, 0x00, 0x84, 0x03, 0x00, 0x00,
            0x00, 0xF1, 0x53, 0x65, 0x00, 0x00, 0x00, 0x00,
            0xB9, 0xBA, 0xBB, 0xBC, 0xBD, 0x88, 0x88, 0x88,
            0x88, 0x88, 0x88, 0x88,
            0x00, 0x00, 0x00, 0x00,
        ];

        let expected = SmaInvLogin {
            src: SmaEndpoint {
                susy_id: 0x5678,
                serial: 0xABCDABCE,
            },
            dst: SmaEndpoint::dummy(),
            counters: SmaInvCounter {
                packet_id: 2,
                ..Default::default()
            },
            timestamp: 1700000000,
            error_code: 1,
            password: Some(SmaInvLogin::pw_from_str("12345").unwrap()),
            ..Default::default()
        };

        let mut cursor = Cursor::new(&serialized[..]);
        match SmaInvLogin::deserialize(&mut cursor) {
            Err(e) => panic!("SmaInvLogin deserialization failed: {e:?}"),
            Ok(message) => {
                assert_eq!(expected, message);
                assert_eq!(SmaInvLogin::LENGTH_MAX, cursor.position());
            }
        }
    }
}
