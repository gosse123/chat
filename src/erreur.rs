use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChatErreur {
    #[error("Erreur d'entrée/sortie : {0}")]
    Io(#[from] io::Error),
    #[error("Erreur de parsage d'adresse : {0}")]
    AddrParse(#[from] std::net::AddrParseError),
    #[error("Erreur de sérialisation : {0}")]
    Json(#[from] serde_json::Error),
    #[error("Erreur de chiffrement : {0}")]
    Encryption(String),
    #[error("Erreur de déchiffrement : {0}")]
    Decryption(String),
    #[error("Message vide")]
    EmptyMessage,
    #[error("Erreur d'historique : {0}")]
    History(String),
    #[error("Clé de chiffrement invalide : {0}")]
    InvalidKey(String),
}
