use std::sync::Arc;

use tokio::net::UdpSocket;
use tokio::sync::mpsc;

use crate::models::message::NetworkMessage;
use crate::security::CryptoEngine;

pub async fn receiver_loop(
    socket: Arc<UdpSocket>,
    crypto: Option<Arc<CryptoEngine>>,
    tx: mpsc::Sender<NetworkMessage>,
) {
    let mut buf = [0u8; 65535];
    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, _addr)) => {
                let data = &buf[..len];
                let plaintext = if let Some(ref crypto) = crypto {
                    match crypto.decrypt(data) {
                        Ok(pt) => pt,
                        Err(e) => {
                            eprintln!("Erreur de déchiffrement: {}", e);
                            continue;
                        }
                    }
                } else {
                    data.to_vec()
                };
                match serde_json::from_slice::<NetworkMessage>(&plaintext) {
                    Ok(msg) => {
                        if tx.send(msg).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Erreur de désérialisation: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Erreur de réception: {}", e);
            }
        }
    }
}
