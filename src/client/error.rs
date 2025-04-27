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

use crate::inverter::{InvalidPasswordError, SmaInvCounter};

/// Errors returned from SMA speedwire client.
#[derive(Clone, Debug)]
pub enum ClientError {
    /// A SMA speedwire protocol error.
    ProtocolError(crate::Error),
    /// An operating system IO error.
    IoError(std::io::ErrorKind),
    /// An operating system clock error.
    TimeError(std::time::SystemTimeError),
    /// The SMA device returned an error.
    DeviceError(u16),
    /// An additional start of fragment packet was received.
    ExtraSofPacket(SmaInvCounter),
    /// Login was rejected by the device.
    LoginFailed,
    /// Invalid input password error.
    InvalidPasswordError(InvalidPasswordError),
}

impl From<std::io::Error> for ClientError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e.kind())
    }
}

impl From<std::time::SystemTimeError> for ClientError {
    fn from(e: std::time::SystemTimeError) -> Self {
        Self::TimeError(e)
    }
}

impl From<crate::Error> for ClientError {
    fn from(e: crate::Error) -> Self {
        Self::ProtocolError(e)
    }
}

impl From<InvalidPasswordError> for ClientError {
    fn from(e: InvalidPasswordError) -> Self {
        Self::InvalidPasswordError(e)
    }
}

impl core::fmt::Display for ClientError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::IoError(e) => {
                write!(f, "{e}")
            }
            Self::TimeError(e) => {
                write!(f, "{e}")
            }
            Self::ProtocolError(e) => {
                write!(f, "{e}")
            }
            Self::DeviceError(ec) => {
                write!(f, "The SMA device returned error code {ec:X}")
            }
            Self::ExtraSofPacket(counter) => {
                write!(
                    f,
                    "Received additional start fragment {}:{}",
                    counter.packet_id, counter.fragment_id
                )
            }
            Self::LoginFailed => {
                write!(f, "The supplied password was rejected")
            }
            Self::InvalidPasswordError(e) => {
                write!(f, "{e}")
            }
        }
    }
}
