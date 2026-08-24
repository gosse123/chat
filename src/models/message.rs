use serde::{Deserialize,Serialize};
use chrono::{DateTime,Utc};

#[derive(Debug,Deserialize,Serialize)]
pub struct Message{
    pseudo: String,
    contenu: String,
    time: DateTime<Utc>,
}