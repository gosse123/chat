mod app;
mod erreur;
mod history;
mod models;
mod network;
mod security;
use crate::erreur::ChatErreur;

use crate::network::{recever, sender};

#[tokio::main]
async fn main() -> Result<(), ChatErreur> {
    recever::recerver().await?;
    sender::sender().await?;
    Ok(())
}
