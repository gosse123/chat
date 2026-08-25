use crate::erreur::ChatErreur;
use std::net::Ipv4Addr;
use tokio::net::UdpSocket;

pub async fn recerver() -> Result<(), ChatErreur> {
    let multicast_addr: Ipv4Addr = "239.0.0.1".parse()?;
    let port = 6000;

    // Écouter sur toutes les interfaces locales sur le port 6000
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).await?;
    println!("📡 Récepteur démarré sur le port {}...", port);

    // Rejoindre le club privé (groupe multicast)
    socket.join_multicast_v4(multicast_addr, Ipv4Addr::UNSPECIFIED)?;
    println!(
        "👥 Groupe multicast {} rejoint avec succès.",
        multicast_addr
    );

    let mut buf = [0u8; 1024];

    // Boucle infinie pour recevoir les messages de manière asynchrone
    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;
        let message = String::from_utf8_lossy(&buf[..len]);
        println!("📩 Reçu de {}: {}", addr, message);
    }
}
