use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChatErreur {
    #[error("Erreur d'entrée/sortie : {0}")]
    Io(#[from] io::Error),
    #[error("Erreur de parsage : {0}")]
    PasseError(#[from] std::net::AddrParseError),
    #[error("Erreur de serialisation")]
    SerdeError(#[from] serde_json::Error),
}
