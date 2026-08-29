use chrono::Local;

use crate::models::message::{Message, NetworkMessage};

pub fn display_network_message(msg: &NetworkMessage) {
    match msg {
        NetworkMessage::Message {
            pseudo,
            content,
            timestamp,
        } => {
            let time = timestamp.with_timezone(&Local);
            println!("[{}] {} : {}", time.format("%H:%M:%S"), pseudo, content);
        }
        NetworkMessage::Discovery { pseudo } => {
            println!(
                "[{}] {} est en ligne",
                Local::now().format("%H:%M:%S"),
                pseudo
            );
        }
        NetworkMessage::Leave { pseudo } => {
            println!(
                "[{}] {} a quitté le chat",
                Local::now().format("%H:%M:%S"),
                pseudo
            );
        }
    }
}

pub fn display_error(msg: &str) {
    eprintln!("Erreur: {}", msg);
}

pub fn display_history(messages: Vec<&Message>) {
    if messages.is_empty() {
        println!("Aucun message dans l'historique.");
        return;
    }
    println!("--- Historique ({} messages) ---", messages.len());
    for msg in &messages {
        let time = msg.time.with_timezone(&Local);
        println!(
            "[{}] {} : {}",
            time.format("%H:%M:%S"),
            msg.pseudo,
            msg.contenu
        );
    }
    println!("--- Fin de l'historique ---");
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_message(pseudo: &str, contenu: &str) -> Message {
        Message {
            pseudo: pseudo.to_string(),
            contenu: contenu.to_string(),
            time: Utc::now(),
        }
    }

    #[test]
    fn test_display_network_message_does_not_panic() {
        let msg = NetworkMessage::Message {
            pseudo: "Alice".to_string(),
            content: "Bonjour".to_string(),
            timestamp: Utc::now(),
        };
        display_network_message(&msg);
    }

    #[test]
    fn test_display_discovery_does_not_panic() {
        let msg = NetworkMessage::Discovery {
            pseudo: "Bob".to_string(),
        };
        display_network_message(&msg);
    }

    #[test]
    fn test_display_leave_does_not_panic() {
        let msg = NetworkMessage::Leave {
            pseudo: "Charlie".to_string(),
        };
        display_network_message(&msg);
    }

    #[test]
    fn test_display_error_does_not_panic() {
        display_error("Test error");
    }

    #[test]
    fn test_display_history_empty() {
        let messages: Vec<&Message> = vec![];
        display_history(messages);
    }

    #[test]
    fn test_display_history_with_messages() {
        let m1 = make_message("Alice", "Bonjour");
        let m2 = make_message("Bob", "Salut");
        let messages: Vec<&Message> = vec![&m1, &m2];
        display_history(messages);
    }
}
