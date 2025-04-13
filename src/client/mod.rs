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

//! High level tokio based SMA speedwire client.

use std::time::SystemTime;

use super::{
    energymeter::{ObisValue, SmaEmMessage},
    inverter::{
        SmaInvCounter, SmaInvGetDayData, SmaInvIdentify, SmaInvLogin,
        SmaInvLogout, SmaInvMeterValue,
    },
    packet::SmaSerde,
    AnySmaMessage, Error, SmaEndpoint,
};

mod error;
mod session;

pub use error::ClientError;
pub use session::SmaSession;

/// SMA client instance for communication with devices.
/// This object holds the network independent communication state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SmaClient {
    /// Client SMA endpoint ID.
    endpoint: SmaEndpoint,
    /// Current packet number.
    packet_id: u16,
}

impl SmaClient {
    /// Creates a new SmaClient with the given SmaEndpoint as source ID.
    pub fn new(endpoint: SmaEndpoint) -> Self {
        Self {
            endpoint,
            packet_id: 0,
        }
    }

    /// Sends an identity request to an SMA device.
    /// Returns the [`SmaEndpoint`] at the clients target IPv4 address.
    pub async fn identify(
        &mut self,
        session: &SmaSession,
    ) -> Result<SmaEndpoint, ClientError> {
        let req = SmaInvIdentify {
            dst: SmaEndpoint::broadcast(),
            src: self.endpoint.clone(),
            counters: self.next_packet(),
            ..Default::default()
        };

        session.write(req).await?;
        let resp = session
            .read(|msg| match msg {
                AnySmaMessage::InvIdentify(resp)
                    if resp.counters.packet_id == self.packet_id =>
                {
                    Some(resp)
                }
                _ => None,
            })
            .await?;

        if resp.error_code != 0 {
            return Err(ClientError::DeviceError(resp.error_code));
        }

        Ok(resp.src)
    }

    /// Sends a login request to an SMA device.
    /// Returns `Ok(())` on successful login or a [`ClientError`] on failure.
    pub async fn login(
        &mut self,
        session: &SmaSession,
        endpoint: &SmaEndpoint,
        passwd: &str,
    ) -> Result<(), ClientError> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs();

        let req = SmaInvLogin {
            dst: endpoint.clone(),
            src: self.endpoint.clone(),
            counters: self.next_packet(),
            timestamp: now as u32,
            password: Some(SmaInvLogin::pw_from_str(passwd)?),
            ..Default::default()
        };

        session.write(req).await?;
        let resp = session
            .read(|msg| match msg {
                AnySmaMessage::InvLogin(resp)
                    if resp.counters.packet_id == self.packet_id =>
                {
                    Some(resp)
                }
                _ => None,
            })
            .await?;

        if resp.error_code != 0 {
            Err(ClientError::LoginFailed)
        } else {
            Ok(())
        }
    }

    /// Sends a logout request to an SMA device.
    /// This command has no response.
    pub async fn logout(
        &mut self,
        session: &SmaSession,
        endpoint: &SmaEndpoint,
    ) -> Result<(), ClientError> {
        let req = SmaInvLogout {
            dst: endpoint.clone(),
            src: self.endpoint.clone(),
            counters: self.next_packet(),
            ..Default::default()
        };

        session.write(req).await
    }

    /// Requests stored energy meter data for a given time range from the
    /// device and returns the received records.
    pub async fn get_day_data(
        &mut self,
        session: &SmaSession,
        endpoint: &SmaEndpoint,
        start_time: u32,
        end_time: u32,
    ) -> Result<Vec<SmaInvMeterValue>, ClientError> {
        let req = SmaInvGetDayData {
            dst: endpoint.clone(),
            src: self.endpoint.clone(),
            counters: self.next_packet(),
            start_time_idx: start_time,
            end_time_idx: end_time,
            ..Default::default()
        };

        session.write(req).await?;

        let mut records = Vec::with_capacity(128);
        let mut total_fragments = 0;
        let mut rx_fragments = 0;
        let mut rx_first = false;

        while rx_fragments != total_fragments || !rx_first {
            let mut resp = session
                .read(|msg| match msg {
                    AnySmaMessage::InvGetDayData(resp)
                        if resp.counters.packet_id == self.packet_id =>
                    {
                        Some(resp)
                    }
                    _ => None,
                })
                .await?;

            rx_fragments += 1;
            if resp.counters.first_fragment {
                if !rx_first {
                    total_fragments = resp.counters.fragment_id + 1;
                    rx_first = true;
                } else {
                    return Err(ClientError::ExtraSofPacket(resp.counters));
                }
            }

            if resp.error_code != 0 {
                return Err(ClientError::DeviceError(resp.error_code));
            }

            records.append(&mut resp.records);
        }

        Ok(records)
    }

    /// Receives a single [`SmaEmMessage`] message and returns the
    /// millisecond timestamp and payload of the message.
    pub async fn read_em_message(
        &mut self,
        session: &SmaSession,
        src: &SmaEndpoint,
    ) -> Result<(u32, Vec<ObisValue>), ClientError> {
        let msg = session
            .read(|msg| match msg {
                AnySmaMessage::EmMessage(resp) if resp.src == *src => {
                    Some(resp)
                }
                _ => None,
            })
            .await?;

        Ok((msg.timestamp_ms, msg.payload))
    }

    /// Broadcasts the given payload with the given millisecond timestamp
    /// in a single [`SmaEmMessage`] message.
    pub async fn write_em_message(
        &mut self,
        session: &SmaSession,
        timestamp_ms: u32,
        payload: Vec<ObisValue>,
    ) -> Result<(), ClientError> {
        let msg = SmaEmMessage {
            src: self.endpoint.clone(),
            timestamp_ms,
            payload,
        };

        session.write(msg).await
    }

    /// Returns the next packet counter.
    fn next_packet(&mut self) -> SmaInvCounter {
        self.packet_id += 1;
        if (self.packet_id & SmaInvCounter::FIRST_FRAGMENT_BIT) != 0 {
            self.packet_id = 0;
        }

        SmaInvCounter {
            packet_id: self.packet_id,
            fragment_id: 0,
            first_fragment: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    use tokio::time;

    #[tokio::test]
    #[ignore]
    async fn read_solar_data() {
        let inv_addr = Ipv4Addr::new(192, 168, 5, 1);
        let mut sma_client = SmaClient::new(SmaEndpoint::dummy());

        let session = match SmaSession::open_unicast(inv_addr) {
            Ok(x) => x,
            Err(e) => panic!("Could not open SMA client session: {e:?}"),
        };

        let result = time::timeout(time::Duration::from_secs(10), async {
            let device = match sma_client.identify(&session).await {
                Ok(identity) => {
                    eprintln!(
                        "{} is {:X}, {:X}",
                        inv_addr, identity.susy_id, identity.serial
                    );
                    identity
                }
                Err(e) => panic!("Could not identify SMA device, {e:?}"),
            };

            if let Err(e) = sma_client.logout(&session, &device).await {
                panic!("Logout failed: {e:?}");
            }
            if let Err(e) = sma_client.login(&session, &device, "0000").await {
                panic!("Login failed: {e:?}");
            }

            let to =
                match SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                    Ok(x) => x.as_secs() as u32,
                    Err(e) => panic!("Getting system time failed: {e:?}"),
                };
            let from = to - 36000;

            eprintln!("GetDayData from {} to {}", from, to);
            match sma_client.get_day_data(&session, &device, from, to).await {
                Err(e) => panic!("Get Day Data failed: {e:?}"),
                Ok(data) => {
                    eprintln!("Get Day data returned {data:?}");
                    eprintln!("Get Day data received {} values", data.len());
                }
            };

            if let Err(e) = sma_client.logout(&session, &device).await {
                panic!("Logout failed: {e:?}");
            }
        })
        .await;

        if result.is_err() {
            panic!("Read solar data test timed out");
        }
    }

    #[tokio::test]
    #[ignore]
    async fn echo_em_message() {
        let mut sma_client = SmaClient::new(SmaEndpoint::dummy());

        let session =
            match SmaSession::open_multicast(Ipv4Addr::new(192, 168, 5, 1)) {
                Ok(x) => x,
                Err(e) => panic!("Could not open SMA client session: {e:?}"),
            };

        let result = time::timeout(time::Duration::from_secs(10), async {
            let (timestamp_ms, payload) = match sma_client
                .read_em_message(
                    &session,
                    &SmaEndpoint {
                        susy_id: 0x015d,
                        serial: 1901439139,
                    },
                )
                .await
            {
                Ok(msg) => {
                    eprintln!("Received: {msg:?}");
                    msg
                }
                Err(e) => panic!("Reading energymeter message failed: {e:?}"),
            };

            if let Err(e) = sma_client
                .write_em_message(&session, timestamp_ms, payload)
                .await
            {
                panic!("Broadcasting energymeter message failed: {e:?}");
            }
        })
        .await;

        if result.is_err() {
            panic!("Read energymeter message test timed out");
        }
    }
}
