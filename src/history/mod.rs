use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

use crate::erreur::ChatErreur;
use crate::models::message::Message;

const MAX_MESSAGES: usize = 100;

pub struct History {
    messages: VecDeque<Message>,
    path: PathBuf,
}

impl History {
    pub fn new(path: PathBuf) -> Self {
        Self {
            messages: VecDeque::new(),
            path,
        }
    }

    pub fn load(&mut self) -> Result<(), ChatErreur> {
        if self.path.exists() {
            let data =
                fs::read_to_string(&self.path).map_err(|e| ChatErreur::History(e.to_string()))?;
            if !data.trim().is_empty() {
                let messages: Vec<Message> = serde_json::from_str(&data)
                    .map_err(|e| ChatErreur::History(e.to_string()))?;
                self.messages = messages.into();
            }
        }
        Ok(())
    }

    pub fn save(&self) -> Result<(), ChatErreur> {
        let data = serde_json::to_string_pretty(&self.messages)
            .map_err(|e| ChatErreur::History(e.to_string()))?;
        fs::write(&self.path, data).map_err(|e| ChatErreur::History(e.to_string()))?;
        Ok(())
    }

    pub fn add(&mut self, message: Message) {
        if self.messages.len() >= MAX_MESSAGES {
            self.messages.pop_front();
        }
        self.messages.push_back(message);
    }

    pub fn get_recent(&self, n: usize) -> Vec<&Message> {
        let start = self.messages.len().saturating_sub(n);
        self.messages.range(start..).collect()
    }
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
    fn test_new_creates_empty_history() {
        let history = History::new(PathBuf::from("test.json"));
        assert!(history.messages.is_empty());
    }

    #[test]
    fn test_add_single_message() {
        let mut history = History::new(PathBuf::from("test.json"));
        history.add(make_message("Alice", "Bonjour"));
        assert_eq!(history.messages.len(), 1);
    }

    #[test]
    fn test_add_multiple_messages() {
        let mut history = History::new(PathBuf::from("test.json"));
        history.add(make_message("Alice", "1"));
        history.add(make_message("Bob", "2"));
        history.add(make_message("Charlie", "3"));
        assert_eq!(history.messages.len(), 3);
    }

    #[test]
    fn test_add_evicts_oldest_when_full() {
        let mut history = History::new(PathBuf::from("test.json"));
        for i in 0..101 {
            history.add(make_message("User", &format!("Message {}", i)));
        }
        assert_eq!(history.messages.len(), 100);
        assert_eq!(history.messages.front().unwrap().contenu, "Message 1");
        assert_eq!(history.messages.back().unwrap().contenu, "Message 100");
    }

    #[test]
    fn test_add_exactly_at_limit() {
        let mut history = History::new(PathBuf::from("test.json"));
        for i in 0..100 {
            history.add(make_message("User", &format!("Message {}", i)));
        }
        assert_eq!(history.messages.len(), 100);
        assert_eq!(history.messages.front().unwrap().contenu, "Message 0");

        history.add(make_message("User", "Message 100"));
        assert_eq!(history.messages.len(), 100);
        assert_eq!(history.messages.front().unwrap().contenu, "Message 1");
    }

    #[test]
    fn test_get_recent_empty_history() {
        let history = History::new(PathBuf::from("test.json"));
        let recent = history.get_recent(10);
        assert!(recent.is_empty());
    }

    #[test]
    fn test_get_recent_zero() {
        let mut history = History::new(PathBuf::from("test.json"));
        history.add(make_message("Alice", "Bonjour"));
        let recent = history.get_recent(0);
        assert!(recent.is_empty());
    }

    #[test]
    fn test_get_recent_more_than_available() {
        let mut history = History::new(PathBuf::from("test.json"));
        history.add(make_message("Alice", "1"));
        history.add(make_message("Bob", "2"));
        let recent = history.get_recent(10);
        assert_eq!(recent.len(), 2);
    }

    #[test]
    fn test_get_recent_exact_count() {
        let mut history = History::new(PathBuf::from("test.json"));
        for i in 0..5 {
            history.add(make_message("User", &format!("Msg {}", i)));
        }
        let recent = history.get_recent(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].contenu, "Msg 2");
        assert_eq!(recent[1].contenu, "Msg 3");
        assert_eq!(recent[2].contenu, "Msg 4");
    }

    #[test]
    fn test_get_recent_preserves_order() {
        let mut history = History::new(PathBuf::from("test.json"));
        history.add(make_message("Alice", "Premier"));
        history.add(make_message("Bob", "Deuxième"));
        history.add(make_message("Charlie", "Troisième"));
        let recent = history.get_recent(2);
        assert_eq!(recent[0].pseudo, "Bob");
        assert_eq!(recent[1].pseudo, "Charlie");
    }
}
