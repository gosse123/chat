use chrono::Utc;
use tempfile::tempdir;

use chat::erreur::ChatErreur;
use chat::history::History;
use chat::models::message::Message;

fn make_message(pseudo: &str, contenu: &str) -> Message {
    Message {
        pseudo: pseudo.to_string(),
        contenu: contenu.to_string(),
        time: Utc::now(),
    }
}

#[test]
fn test_save_and_load_roundtrip() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("history.json");

    {
        let mut history = History::new(path.clone());
        history.add(make_message("Alice", "Bonjour"));
        history.add(make_message("Bob", "Salut"));
        history.save().unwrap();
    }

    let mut history = History::new(path);
    history.load().unwrap();
    let messages = history.get_recent(10);
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].pseudo, "Alice");
    assert_eq!(messages[0].contenu, "Bonjour");
    assert_eq!(messages[1].pseudo, "Bob");
    assert_eq!(messages[1].contenu, "Salut");
}

#[test]
fn test_load_nonexistent_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("nonexistent.json");

    let mut history = History::new(path);
    let result = history.load();
    assert!(result.is_ok());
    assert!(history.get_recent(10).is_empty());
}

#[test]
fn test_load_empty_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("empty.json");
    std::fs::write(&path, "").unwrap();

    let mut history = History::new(path);
    let result = history.load();
    assert!(result.is_ok());
    assert!(history.get_recent(10).is_empty());
}

#[test]
fn test_load_whitespace_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("whitespace.json");
    std::fs::write(&path, "   \n\t  ").unwrap();

    let mut history = History::new(path);
    let result = history.load();
    assert!(result.is_ok());
}

#[test]
fn test_load_invalid_json() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("invalid.json");
    std::fs::write(&path, "not valid json{{{").unwrap();

    let mut history = History::new(path);
    let result = history.load();
    assert!(result.is_err());
    match result.unwrap_err() {
        ChatErreur::History(_) => {}
        e => panic!("Expected History error, got {:?}", e),
    }
}

#[test]
fn test_save_preserves_order() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("history.json");

    {
        let mut history = History::new(path.clone());
        for i in 0..10 {
            history.add(make_message("User", &format!("Message {}", i)));
        }
        history.save().unwrap();
    }

    let mut history = History::new(path);
    history.load().unwrap();
    let messages = history.get_recent(10);
    for i in 0..10 {
        assert_eq!(messages[i].contenu, format!("Message {}", i));
    }
}

#[test]
fn test_save_load_many_messages() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("history.json");

    {
        let mut history = History::new(path.clone());
        for i in 0..100 {
            history.add(make_message("User", &format!("Msg {}", i)));
        }
        history.save().unwrap();
    }

    let mut history = History::new(path);
    history.load().unwrap();
    let messages = history.get_recent(100);
    assert_eq!(messages.len(), 100);
    assert_eq!(messages[0].contenu, "Msg 0");
    assert_eq!(messages[99].contenu, "Msg 99");
}

#[test]
fn test_save_load_unicode() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("history.json");

    {
        let mut history = History::new(path.clone());
        history.add(make_message("Aliçe", "日本語テスト"));
        history.save().unwrap();
    }

    let mut history = History::new(path);
    history.load().unwrap();
    let messages = history.get_recent(10);
    assert_eq!(messages[0].pseudo, "Aliçe");
    assert_eq!(messages[0].contenu, "日本語テスト");
}

#[test]
fn test_overwrite_existing_history() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("history.json");

    {
        let mut history = History::new(path.clone());
        history.add(make_message("Alice", "Old message"));
        history.save().unwrap();
    }

    {
        let mut history = History::new(path.clone());
        history.add(make_message("Bob", "New message"));
        history.save().unwrap();
    }

    let mut history = History::new(path);
    history.load().unwrap();
    let messages = history.get_recent(10);
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].pseudo, "Bob");
    assert_eq!(messages[0].contenu, "New message");
}
