use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::UdpSocket;

use crate::erreur::ChatErreur;
use crate::models::message::NetworkMessage;
use crate::security::CryptoEngine;

pub struct MulticastSender {
    socket: Arc<UdpSocket>,
    addr: SocketAddr,
    crypto: Option<Arc<CryptoEngine>>,
}

impl MulticastSender {
    pub fn new(
        socket: Arc<UdpSocket>,
        addr: SocketAddr,
        crypto: Option<Arc<CryptoEngine>>,
    ) -> Self {
        Self {
            socket,
            addr,
            crypto,
        }
    }

    pub async fn send(&self, data: &[u8]) -> Result<(), ChatErreur> {
        self.socket.send_to(data, self.addr).await?;
        Ok(())
    }

    pub async fn send_message(&self, msg: &NetworkMessage) -> Result<(), ChatErreur> {
        let json = serde_json::to_vec(msg)?;
        let data = if let Some(ref crypto) = self.crypto {
            crypto.encrypt(&json)?
        } else {
            json
        };
        self.send(&data).await
    }
}
