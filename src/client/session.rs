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

use super::{AnySmaMessage, ClientError, Cursor, Error, SmaSerde};

// Required for set_multicast_if_v4 and set_reuse_address
use socket2::{Domain, Socket, Type};
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::net::UdpSocket;

/// SMA client session instance that holds the network dependent state
/// for communication with a single unicast device, or a group of multicast
/// devices.
#[derive(Debug)]
pub struct SmaSession {
    multicast: bool,
    dst_sockaddr: SocketAddrV4,
    socket: UdpSocket,
}

impl SmaSession {
    /// Largest seen SMA speedwire packet size before fragmentation.
    const BUFFER_SIZE: usize = 1030;

    const SMA_PORT: u16 = 9522;
    const SMA_MCAST_ADDR: Ipv4Addr = Ipv4Addr::new(239, 12, 255, 254);

    /// Opens a unicast network socket for communication with a single SMA
    /// device identified by a IP address.
    pub fn open_unicast(remote_addr: Ipv4Addr) -> Result<Self, ClientError> {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, None)?;
        socket.bind(&SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0).into())?;
        socket.set_nonblocking(true)?;

        Ok(Self {
            multicast: false,
            socket: UdpSocket::from_std(socket.into())?,
            dst_sockaddr: SocketAddrV4::new(remote_addr, Self::SMA_PORT),
        })
    }

    /// Opens a multicast network socket on the given local IPv4 address for
    /// communication with a group of SMA devices.
    pub fn open_multicast(local_addr: Ipv4Addr) -> Result<Self, ClientError> {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, None)?;
        socket.set_reuse_address(true)?;
        socket.bind(
            &SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), Self::SMA_PORT)
                .into(),
        )?;
        socket.set_nonblocking(true)?;

        socket.set_multicast_loop_v4(false)?;
        socket.set_multicast_if_v4(&local_addr)?;
        socket.join_multicast_v4(&Self::SMA_MCAST_ADDR, &local_addr)?;

        Ok(Self {
            multicast: true,
            socket: UdpSocket::from_std(socket.into())?,
            dst_sockaddr: SocketAddrV4::new(
                Self::SMA_MCAST_ADDR,
                Self::SMA_PORT,
            ),
        })
    }

    pub(crate) async fn write<T: SmaSerde>(
        &self,
        msg: T,
    ) -> Result<(), ClientError> {
        let mut buffer = [0u8; Self::BUFFER_SIZE];
        let mut cursor = Cursor::new(&mut buffer[..]);

        msg.serialize(&mut cursor)?;
        let len = cursor.position();

        Ok(self
            .socket
            .send_to(&buffer[..len], self.dst_sockaddr)
            .await
            .map(|_| ())?)
    }

    pub(crate) async fn read<T: SmaSerde>(
        &self,
        predicate: impl Fn(AnySmaMessage) -> Option<T>,
    ) -> Result<T, ClientError> {
        let mut buffer = [0u8; Self::BUFFER_SIZE];

        loop {
            let (rx_len, rx_addr) = self.socket.recv_from(&mut buffer).await?;

            if self.multicast || rx_addr.ip() == *self.dst_sockaddr.ip() {
                // Since speedwire is a multicast protocol, receiving an
                // incorrect message type is not necessarily an
                // error as it could be just another broadcast message.
                let mut cursor = Cursor::new(&buffer[..rx_len]);
                let message = match AnySmaMessage::deserialize(&mut cursor) {
                    Ok(x) => x,
                    // Ignore unknown SMA protocols in multicast mode.
                    Err(Error::UnsupportedProtocol { .. })
                        if self.multicast =>
                    {
                        continue
                    }
                    Err(e) => return Err(e.into()),
                };

                if let Some(x) = predicate(message) {
                    return Ok(x);
                }
            }
        }
    }
}
