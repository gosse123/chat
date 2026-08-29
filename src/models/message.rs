use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub pseudo: String,
    pub contenu: String,
    pub time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NetworkMessage {
    #[serde(rename = "message")]
    Message {
        pseudo: String,
        content: String,
        timestamp: DateTime<Utc>,
    },
    #[serde(rename = "discovery")]
    Discovery { pseudo: String },
    #[serde(rename = "leave")]
    Leave { pseudo: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_time() -> DateTime<Utc> {
        "2026-08-24T14:23:45Z".parse().unwrap()
    }

    #[test]
    fn test_message_serde_roundtrip() {
        let msg = Message {
            pseudo: "Alice".to_string(),
            contenu: "Bonjour tout le monde !".to_string(),
            time: test_time(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.pseudo, "Alice");
        assert_eq!(deserialized.contenu, "Bonjour tout le monde !");
        assert_eq!(deserialized.time, test_time());
    }

    #[test]
    fn test_network_message_message_serde_roundtrip() {
        let msg = NetworkMessage::Message {
            pseudo: "Alice".to_string(),
            content: "Bonjour !".to_string(),
            timestamp: test_time(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: NetworkMessage = serde_json::from_str(&json).unwrap();

        match deserialized {
            NetworkMessage::Message {
                pseudo,
                content,
                timestamp,
            } => {
                assert_eq!(pseudo, "Alice");
                assert_eq!(content, "Bonjour !");
                assert_eq!(timestamp, test_time());
            }
            _ => panic!("Expected NetworkMessage::Message"),
        }
    }

    #[test]
    fn test_network_message_message_has_type_tag() {
        let msg = NetworkMessage::Message {
            pseudo: "Bob".to_string(),
            content: "Salut".to_string(),
            timestamp: test_time(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"message""#));
    }

    #[test]
    fn test_network_message_discovery_serde_roundtrip() {
        let msg = NetworkMessage::Discovery {
            pseudo: "Charlie".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: NetworkMessage = serde_json::from_str(&json).unwrap();

        match deserialized {
            NetworkMessage::Discovery { pseudo } => {
                assert_eq!(pseudo, "Charlie");
            }
            _ => panic!("Expected NetworkMessage::Discovery"),
        }
    }

    #[test]
    fn test_network_message_discovery_has_type_tag() {
        let msg = NetworkMessage::Discovery {
            pseudo: "Dave".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"discovery""#));
    }

    #[test]
    fn test_network_message_leave_serde_roundtrip() {
        let msg = NetworkMessage::Leave {
            pseudo: "Eve".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: NetworkMessage = serde_json::from_str(&json).unwrap();

        match deserialized {
            NetworkMessage::Leave { pseudo } => {
                assert_eq!(pseudo, "Eve");
            }
            _ => panic!("Expected NetworkMessage::Leave"),
        }
    }

    #[test]
    fn test_network_message_leave_has_type_tag() {
        let msg = NetworkMessage::Leave {
            pseudo: "Frank".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"leave""#));
    }

    #[test]
    fn test_network_message_json_format() {
        let msg = NetworkMessage::Message {
            pseudo: "Alice".to_string(),
            content: "Bonjour tout le monde !".to_string(),
            timestamp: test_time(),
        };
        let json = serde_json::to_string_pretty(&msg).unwrap();
        assert!(json.contains(r#""type": "message""#));
        assert!(json.contains(r#""pseudo": "Alice""#));
        assert!(json.contains(r#""content": "Bonjour tout le monde !""#));
        assert!(json.contains(r#""timestamp": "2026-08-24T14:23:45Z""#));
    }

    #[test]
    fn test_network_message_clone() {
        let msg = NetworkMessage::Message {
            pseudo: "Alice".to_string(),
            content: "Test".to_string(),
            timestamp: test_time(),
        };
        let cloned = msg.clone();
        match cloned {
            NetworkMessage::Message { pseudo, .. } => assert_eq!(pseudo, "Alice"),
            _ => panic!("Clone failed"),
        }
    }

    #[test]
    fn test_message_empty_content() {
        let msg = Message {
            pseudo: "Alice".to_string(),
            contenu: String::new(),
            time: test_time(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.contenu, "");
    }

    #[test]
    fn test_network_message_unicode() {
        let msg = NetworkMessage::Message {
            pseudo: "Aliçe".to_string(),
            content: "こんにちは".to_string(),
            timestamp: test_time(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: NetworkMessage = serde_json::from_str(&json).unwrap();
        match deserialized {
            NetworkMessage::Message { pseudo, content, .. } => {
                assert_eq!(pseudo, "Aliçe");
                assert_eq!(content, "こんにちは");
            }
            _ => panic!("Expected NetworkMessage::Message"),
        }
    }
}
