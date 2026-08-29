use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::Utc;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

use crate::erreur::ChatErreur;
use crate::history::History;
use crate::models::message::{Message, NetworkMessage};
use crate::network::recever;
use crate::network::sender::MulticastSender;
use crate::security::CryptoEngine;
use crate::ui;

const MULTICAST_ADDR: &str = "239.0.0.1:5000";

pub struct Chat {
    pseudo: String,
    socket: Arc<UdpSocket>,
    sender: MulticastSender,
    crypto: Option<Arc<CryptoEngine>>,
    history: History,
}

impl Chat {
    pub async fn new(pseudo: String, crypto_key: Option<String>) -> Result<Self, ChatErreur> {
        let socket = Self::create_multicast_socket()?;
        let socket = Arc::new(socket);

        let multicast_addr: SocketAddr = MULTICAST_ADDR.parse()?;

        let crypto = match crypto_key {
            Some(hex_key) => {
                let key =
                    hex::decode(&hex_key).map_err(|e| ChatErreur::InvalidKey(e.to_string()))?;
                Some(Arc::new(CryptoEngine::new(&key)?))
            }
            None => None,
        };

        let sender = MulticastSender::new(socket.clone(), multicast_addr, crypto.clone());

        let history_path = PathBuf::from("chat_history.json");
        let mut history = History::new(history_path);
        history.load()?;

        Ok(Self {
            pseudo,
            socket,
            sender,
            crypto,
            history,
        })
    }

    fn create_multicast_socket() -> Result<UdpSocket, ChatErreur> {
        use socket2::{Domain, Protocol, Socket, Type};
        use std::net::Ipv4Addr;

        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        socket.set_reuse_address(true)?;

        #[cfg(unix)]
        socket.set_reuse_port(true)?;

        let bind_addr: SocketAddr = "0.0.0.0:5000".parse()?;
        socket.bind(&bind_addr.into())?;

        let multicast_ip: Ipv4Addr = "239.0.0.1".parse().unwrap();
        socket.join_multicast_v4(&multicast_ip, &Ipv4Addr::UNSPECIFIED)?;

        socket.set_multicast_ttl_v4(1)?;
        socket.set_nonblocking(true)?;

        let std_socket: std::net::UdpSocket = socket.into();
        let tokio_socket = UdpSocket::from_std(std_socket)?;

        Ok(tokio_socket)
    }

    pub async fn run(&mut self) -> Result<(), ChatErreur> {
        let (tx, mut rx) = mpsc::channel::<NetworkMessage>(100);

        let recv_socket = self.socket.clone();
        let recv_crypto = self.crypto.clone();
        tokio::spawn(async move {
            recever::receiver_loop(recv_socket, recv_crypto, tx).await;
        });

        self.send_discovery().await?;

        println!("Bienvenue {}! Tapez /help pour les commandes.", self.pseudo);

        let stdin = io::stdin();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();

        loop {
            tokio::select! {
                line = lines.next_line() => {
                    match line {
                        Ok(Some(line)) => {
                            let line = line.trim().to_string();
                            if line.is_empty() {
                                println!("Message vide, réessayez.");
                                continue;
                            }
                            match line.as_str() {
                                "/quit" | "/exit" => {
                                    self.send_leave().await?;
                                    println!("Au revoir!");
                                    break;
                                }
                                "/history" => {
                                    let recent = self.history.get_recent(20);
                                    ui::display_history(recent);
                                }
                                "/help" => {
                                    println!("Commandes:");
                                    println!("  /history - Afficher l'historique");
                                    println!("  /quit    - Quitter le chat");
                                    println!("  /help    - Afficher cette aide");
                                }
                                _ => {
                                    self.send_user_message(&line).await?;
                                }
                            }
                        }
                        _ => break,
                    }
                }
                event = rx.recv() => {
                    if let Some(msg) = event {
                        self.handle_received(&msg).await;
                    } else {
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    async fn send_user_message(&mut self, content: &str) -> Result<(), ChatErreur> {
        let msg = NetworkMessage::Message {
            pseudo: self.pseudo.clone(),
            content: content.to_string(),
            timestamp: Utc::now(),
        };

        self.sender.send_message(&msg).await?;

        let internal = Message {
            pseudo: self.pseudo.clone(),
            contenu: content.to_string(),
            time: Utc::now(),
        };
        self.history.add(internal);
        self.history.save()?;

        Ok(())
    }

    async fn send_discovery(&self) -> Result<(), ChatErreur> {
        let msg = NetworkMessage::Discovery {
            pseudo: self.pseudo.clone(),
        };
        self.sender.send_message(&msg).await
    }

    async fn send_leave(&self) -> Result<(), ChatErreur> {
        let msg = NetworkMessage::Leave {
            pseudo: self.pseudo.clone(),
        };
        self.sender.send_message(&msg).await
    }

    async fn handle_received(&mut self, msg: &NetworkMessage) {
        if let NetworkMessage::Message { pseudo, .. } = msg {
            if pseudo == &self.pseudo {
                return;
            }
        }

        ui::display_network_message(msg);

        if let NetworkMessage::Message {
            pseudo,
            content,
            timestamp,
        } = msg
        {
            let internal = Message {
                pseudo: pseudo.clone(),
                contenu: content.clone(),
                time: *timestamp,
            };
            self.history.add(internal);
            if let Err(e) = self.history.save() {
                eprintln!("Erreur de sauvegarde: {}", e);
            }
        }
    }
}
