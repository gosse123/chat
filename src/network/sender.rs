use std::net::Ipv4Addr;
use tokio::net::UdpSocket;
use tokio::time::{Duration, sleep};

use crate::erreur::ChatErreur;

pub async fn sender() -> Result<(), ChatErreur> {
    let multicast_addr: Ipv4Addr = "239.0.0.1".parse()?;
    let port = 6000;
    let target = (multicast_addr, port);

    // Se lier à un port aléatoire disponible pour envoyer
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    println!(
        "🚀 Émetteur prêt. Envoi de messages vers {}...",
        multicast_addr
    );

    let mut compteur = 1;

    loop {
        let message = format!("Message multicast asynchrone numéro {}", compteur);

        // Envoi du message au groupe
        socket.send_to(message.as_bytes(), target).await?;
        println!("📤 Envoyé : {}", message);

        compteur += 1;
        // Attendre 2 secondes avant le prochain envoi
        sleep(Duration::from_secs(2)).await;
    }
}
