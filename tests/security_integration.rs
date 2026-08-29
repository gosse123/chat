use chat::erreur::ChatErreur;
use chat::models::message::NetworkMessage;
use chat::security::CryptoEngine;

fn test_key() -> [u8; 32] {
    [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
        0x1d, 0x1e, 0x1f, 0x20,
    ]
}

#[test]
fn test_encrypt_then_decrypt_json_message() {
    let engine = CryptoEngine::new(&test_key()).unwrap();

    let msg = NetworkMessage::Message {
        pseudo: "Alice".to_string(),
        content: "Bonjour tout le monde !".to_string(),
        timestamp: "2026-08-24T14:23:45Z".parse().unwrap(),
    };

    let json = serde_json::to_vec(&msg).unwrap();
    let ciphertext = engine.encrypt(&json).unwrap();
    let plaintext = engine.decrypt(&ciphertext).unwrap();
    let received: NetworkMessage = serde_json::from_slice(&plaintext).unwrap();

    match received {
        NetworkMessage::Message {
            pseudo,
            content,
            timestamp,
        } => {
            assert_eq!(pseudo, "Alice");
            assert_eq!(content, "Bonjour tout le monde !");
            assert_eq!(timestamp.to_rfc3339(), "2026-08-24T14:23:45+00:00");
        }
        _ => panic!("Expected NetworkMessage::Message"),
    }
}

#[test]
fn test_encrypt_then_decrypt_discovery() {
    let engine = CryptoEngine::new(&test_key()).unwrap();

    let msg = NetworkMessage::Discovery {
        pseudo: "Bob".to_string(),
    };

    let json = serde_json::to_vec(&msg).unwrap();
    let ciphertext = engine.encrypt(&json).unwrap();
    let plaintext = engine.decrypt(&ciphertext).unwrap();
    let received: NetworkMessage = serde_json::from_slice(&plaintext).unwrap();

    match received {
        NetworkMessage::Discovery { pseudo } => {
            assert_eq!(pseudo, "Bob");
        }
        _ => panic!("Expected NetworkMessage::Discovery"),
    }
}

#[test]
fn test_encrypt_then_decrypt_leave() {
    let engine = CryptoEngine::new(&test_key()).unwrap();

    let msg = NetworkMessage::Leave {
        pseudo: "Charlie".to_string(),
    };

    let json = serde_json::to_vec(&msg).unwrap();
    let ciphertext = engine.encrypt(&json).unwrap();
    let plaintext = engine.decrypt(&ciphertext).unwrap();
    let received: NetworkMessage = serde_json::from_slice(&plaintext).unwrap();

    match received {
        NetworkMessage::Leave { pseudo } => {
            assert_eq!(pseudo, "Charlie");
        }
        _ => panic!("Expected NetworkMessage::Leave"),
    }
}

#[test]
fn test_multiple_encryption_cycles() {
    let engine = CryptoEngine::new(&test_key()).unwrap();

    let messages = vec![
        NetworkMessage::Message {
            pseudo: "Alice".to_string(),
            content: "Premier message".to_string(),
            timestamp: chrono::Utc::now(),
        },
        NetworkMessage::Discovery {
            pseudo: "Bob".to_string(),
        },
        NetworkMessage::Leave {
            pseudo: "Charlie".to_string(),
        },
    ];

    for msg in &messages {
        let json = serde_json::to_vec(msg).unwrap();
        let ciphertext = engine.encrypt(&json).unwrap();
        let plaintext = engine.decrypt(&ciphertext).unwrap();
        let received: NetworkMessage = serde_json::from_slice(&plaintext).unwrap();

        match (msg, &received) {
            (
                NetworkMessage::Message {
                    pseudo: p1,
                    content: c1,
                    ..
                },
                NetworkMessage::Message {
                    pseudo: p2,
                    content: c2,
                    ..
                },
            ) => {
                assert_eq!(p1, p2);
                assert_eq!(c1, c2);
            }
            (
                NetworkMessage::Discovery { pseudo: p1 },
                NetworkMessage::Discovery { pseudo: p2 },
            ) => assert_eq!(p1, p2),
            (
                NetworkMessage::Leave { pseudo: p1 },
                NetworkMessage::Leave { pseudo: p2 },
            ) => assert_eq!(p1, p2),
            _ => panic!("Message type mismatch"),
        }
    }
}

#[test]
fn test_ciphertext_differs_each_time() {
    let engine = CryptoEngine::new(&test_key()).unwrap();
    let plaintext = b"Same message every time";

    let ct1 = engine.encrypt(plaintext).unwrap();
    let ct2 = engine.encrypt(plaintext).unwrap();
    let ct3 = engine.encrypt(plaintext).unwrap();

    assert_ne!(ct1, ct2);
    assert_ne!(ct2, ct3);
    assert_ne!(ct1, ct3);
}

#[test]
fn test_wrong_key_cannot_decrypt() {
    let key1 = test_key();
    let mut key2 = test_key();
    key2[0] ^= 0xff;

    let engine1 = CryptoEngine::new(&key1).unwrap();
    let engine2 = CryptoEngine::new(&key2).unwrap();

    let msg = NetworkMessage::Message {
        pseudo: "Alice".to_string(),
        content: "Secret".to_string(),
        timestamp: chrono::Utc::now(),
    };

    let json = serde_json::to_vec(&msg).unwrap();
    let ciphertext = engine1.encrypt(&json).unwrap();
    let result = engine2.decrypt(&ciphertext);
    assert!(result.is_err());
}

#[test]
fn test_tampered_ciphertext_fails() {
    let engine = CryptoEngine::new(&test_key()).unwrap();

    let msg = NetworkMessage::Message {
        pseudo: "Alice".to_string(),
        content: "Original".to_string(),
        timestamp: chrono::Utc::now(),
    };

    let json = serde_json::to_vec(&msg).unwrap();
    let mut ciphertext = engine.encrypt(&json).unwrap();

    // Tamper with the ciphertext (modify a byte in the encrypted part)
    if ciphertext.len() > 15 {
        ciphertext[15] ^= 0xff;
    }

    let result = engine.decrypt(&ciphertext);
    assert!(result.is_err());
}

#[test]
fn test_large_message_encryption() {
    let engine = CryptoEngine::new(&test_key()).unwrap();

    let large_content = "A".repeat(50000);
    let msg = NetworkMessage::Message {
        pseudo: "Alice".to_string(),
        content: large_content.clone(),
        timestamp: chrono::Utc::now(),
    };

    let json = serde_json::to_vec(&msg).unwrap();
    let ciphertext = engine.encrypt(&json).unwrap();
    let plaintext = engine.decrypt(&ciphertext).unwrap();
    let received: NetworkMessage = serde_json::from_slice(&plaintext).unwrap();

    match received {
        NetworkMessage::Message { content, .. } => {
            assert_eq!(content, large_content);
        }
        _ => panic!("Expected NetworkMessage::Message"),
    }
}

#[test]
fn test_unicode_message_encryption() {
    let engine = CryptoEngine::new(&test_key()).unwrap();

    let msg = NetworkMessage::Message {
        pseudo: "Aliçe".to_string(),
        content: "こんにちは世界 🌍".to_string(),
        timestamp: chrono::Utc::now(),
    };

    let json = serde_json::to_vec(&msg).unwrap();
    let ciphertext = engine.encrypt(&json).unwrap();
    let plaintext = engine.decrypt(&ciphertext).unwrap();
    let received: NetworkMessage = serde_json::from_slice(&plaintext).unwrap();

    match received {
        NetworkMessage::Message {
            pseudo, content, ..
        } => {
            assert_eq!(pseudo, "Aliçe");
            assert_eq!(content, "こんにちは世界 🌍");
        }
        _ => panic!("Expected NetworkMessage::Message"),
    }
}
