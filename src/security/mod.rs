use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::rngs::OsRng;
use rand::RngCore;

use crate::erreur::ChatErreur;

pub struct CryptoEngine {
    cipher: Aes256Gcm,
}

impl std::fmt::Debug for CryptoEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CryptoEngine").finish()
    }
}

impl CryptoEngine {
    pub fn new(key: &[u8]) -> Result<Self, ChatErreur> {
        if key.len() != 32 {
            return Err(ChatErreur::InvalidKey(format!(
                "La clé doit faire 32 octets, {} fournis",
                key.len()
            )));
        }
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| ChatErreur::Encryption(e.to_string()))?;
        Ok(Self { cipher })
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, ChatErreur> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| ChatErreur::Encryption(e.to_string()))?;

        let mut result = nonce_bytes.to_vec();
        result.extend(ciphertext);
        Ok(result)
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, ChatErreur> {
        if data.len() < 12 {
            return Err(ChatErreur::Decryption(
                "Données trop courtes pour contenir un nonce".into(),
            ));
        }
        let nonce = Nonce::from_slice(&data[..12]);
        let ciphertext = &data[12..];

        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| ChatErreur::Decryption(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a,
            0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
        ]
    }

    #[test]
    fn test_new_with_valid_key() {
        let key = test_key();
        let engine = CryptoEngine::new(&key);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_new_with_wrong_length_key() {
        let key = [0u8; 16];
        let result = CryptoEngine::new(&key);
        assert!(result.is_err());
        match result.unwrap_err() {
            ChatErreur::InvalidKey(msg) => {
                assert!(msg.contains("32 octets"));
                assert!(msg.contains("16 fournis"));
            }
            _ => panic!("Expected InvalidKey error"),
        }
    }

    #[test]
    fn test_new_with_empty_key() {
        let result = CryptoEngine::new(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_produces_output() {
        let engine = CryptoEngine::new(&test_key()).unwrap();
        let plaintext = b"Hello, World!";
        let ciphertext = engine.encrypt(plaintext).unwrap();
        assert!(!ciphertext.is_empty());
    }

    #[test]
    fn test_encrypt_output_longer_than_plaintext() {
        let engine = CryptoEngine::new(&test_key()).unwrap();
        let plaintext = b"Test";
        let ciphertext = engine.encrypt(plaintext).unwrap();
        // 12 bytes nonce + 4 bytes plaintext + 16 bytes auth tag = 32 bytes
        assert!(ciphertext.len() >= plaintext.len() + 12 + 16);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let engine = CryptoEngine::new(&test_key()).unwrap();
        let plaintext = b"Bonjour tout le monde!";
        let ciphertext = engine.encrypt(plaintext).unwrap();
        let decrypted = engine.decrypt(&ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_empty_plaintext() {
        let engine = CryptoEngine::new(&test_key()).unwrap();
        let plaintext = b"";
        let ciphertext = engine.encrypt(plaintext).unwrap();
        let decrypted = engine.decrypt(&ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_long_message() {
        let engine = CryptoEngine::new(&test_key()).unwrap();
        let plaintext = vec![0u8; 10000];
        let ciphertext = engine.encrypt(&plaintext).unwrap();
        let decrypted = engine.decrypt(&ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_too_short_data() {
        let engine = CryptoEngine::new(&test_key()).unwrap();
        let result = engine.decrypt(&[0u8; 5]);
        assert!(result.is_err());
        match result.unwrap_err() {
            ChatErreur::Decryption(msg) => {
                assert!(msg.contains("trop courtes"));
            }
            _ => panic!("Expected Decryption error"),
        }
    }

    #[test]
    fn test_decrypt_garbage_data() {
        let engine = CryptoEngine::new(&test_key()).unwrap();
        let garbage = vec![0u8; 100];
        let result = engine.decrypt(&garbage);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_different_nonces() {
        let engine = CryptoEngine::new(&test_key()).unwrap();
        let plaintext = b"Same message";
        let ct1 = engine.encrypt(plaintext).unwrap();
        let ct2 = engine.encrypt(plaintext).unwrap();
        // Nonces are random, so ciphertexts should differ
        assert_ne!(ct1, ct2);
    }

    #[test]
    fn test_wrong_key_fails_decryption() {
        let key1 = test_key();
        let mut key2 = test_key();
        key2[0] ^= 0xff;

        let engine1 = CryptoEngine::new(&key1).unwrap();
        let engine2 = CryptoEngine::new(&key2).unwrap();

        let plaintext = b"Secret message";
        let ciphertext = engine1.encrypt(plaintext).unwrap();
        let result = engine2.decrypt(&ciphertext);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_json_message() {
        let engine = CryptoEngine::new(&test_key()).unwrap();
        let json = r#"{"type":"message","pseudo":"Alice","content":"Bonjour","timestamp":"2026-08-24T14:23:45Z"}"#;
        let ciphertext = engine.encrypt(json.as_bytes()).unwrap();
        let decrypted = engine.decrypt(&ciphertext).unwrap();
        assert_eq!(String::from_utf8(decrypted).unwrap(), json);
    }
}
