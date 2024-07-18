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

/// Errors returned from SMA speedwire client.
#[derive(Clone, Debug)]
pub enum ClientError {
    /// A SMA speedwire protocol error.
    ProtocolError(crate::Error),
    /// An operating system IO error.
    IoError(std::io::ErrorKind),
}

impl From<crate::Error> for ClientError {
    fn from(e: crate::Error) -> Self {
        Self::ProtocolError(e)
    }
}

impl From<std::io::Error> for ClientError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e.kind())
    }
}
