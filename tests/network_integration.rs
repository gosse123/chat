use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use chat::models::message::NetworkMessage;
use chat::network::recever;
use chat::network::sender::MulticastSender;
use chat::security::CryptoEngine;

async fn create_test_socket(port: u16) -> Arc<UdpSocket> {
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
    let socket = UdpSocket::bind(addr).await.unwrap();
    Arc::new(socket)
}

fn test_key() -> [u8; 32] {
    [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
        0x1d, 0x1e, 0x1f, 0x20,
    ]
}

#[tokio::test]
async fn test_sender_send_raw_bytes() {
    let receiver = create_test_socket(19001).await;
    let sender = create_test_socket(19002).await;
    let addr: SocketAddr = "127.0.0.1:19001".parse().unwrap();

    let multicast_sender = MulticastSender::new(sender, addr, None);

    let data = b"Hello, UDP!";
    multicast_sender.send(data).await.unwrap();

    let mut buf = [0u8; 1024];
    let (len, _) = receiver.recv_from(&mut buf).await.unwrap();
    assert_eq!(&buf[..len], data);
}

#[tokio::test]
async fn test_sender_send_message_without_crypto() {
    let receiver = create_test_socket(19003).await;
    let sender = create_test_socket(19004).await;
    let addr: SocketAddr = "127.0.0.1:19003".parse().unwrap();

    let multicast_sender = MulticastSender::new(sender, addr, None);

    let msg = NetworkMessage::Message {
        pseudo: "Alice".to_string(),
        content: "Bonjour".to_string(),
        timestamp: chrono::Utc::now(),
    };

    multicast_sender.send_message(&msg).await.unwrap();

    let mut buf = [0u8; 65535];
    let (len, _) = receiver.recv_from(&mut buf).await.unwrap();
    let received: NetworkMessage = serde_json::from_slice(&buf[..len]).unwrap();

    match received {
        NetworkMessage::Message { pseudo, content, .. } => {
            assert_eq!(pseudo, "Alice");
            assert_eq!(content, "Bonjour");
        }
        _ => panic!("Expected NetworkMessage::Message"),
    }
}

#[tokio::test]
async fn test_sender_send_message_with_crypto() {
    let receiver = create_test_socket(19005).await;
    let sender = create_test_socket(19006).await;
    let addr: SocketAddr = "127.0.0.1:19005".parse().unwrap();

    let key = test_key();
    let crypto = Some(Arc::new(CryptoEngine::new(&key).unwrap()));
    let multicast_sender = MulticastSender::new(sender, addr, crypto);

    let msg = NetworkMessage::Discovery {
        pseudo: "Bob".to_string(),
    };

    multicast_sender.send_message(&msg).await.unwrap();

    let mut buf = [0u8; 65535];
    let (len, _) = receiver.recv_from(&mut buf).await.unwrap();
    assert_ne!(&buf[..len], b"{\"type\":\"discovery\",\"pseudo\":\"Bob\"}");

    let crypto2 = CryptoEngine::new(&test_key()).unwrap();
    let plaintext = crypto2.decrypt(&buf[..len]).unwrap();
    let received: NetworkMessage = serde_json::from_slice(&plaintext).unwrap();

    match received {
        NetworkMessage::Discovery { pseudo } => {
            assert_eq!(pseudo, "Bob");
        }
        _ => panic!("Expected NetworkMessage::Discovery"),
    }
}

#[tokio::test]
async fn test_receiver_loop_receives_messages() {
    let receiver = create_test_socket(19007).await;
    let sender = create_test_socket(19008).await;
    let addr: SocketAddr = "127.0.0.1:19007".parse().unwrap();

    let (tx, mut rx) = mpsc::channel::<NetworkMessage>(10);
    let recv_socket = receiver.clone();

    tokio::spawn(async move {
        recever::receiver_loop(recv_socket, None, tx).await;
    });

    let multicast_sender = MulticastSender::new(sender, addr, None);

    let msg = NetworkMessage::Message {
        pseudo: "Alice".to_string(),
        content: "Test".to_string(),
        timestamp: chrono::Utc::now(),
    };

    multicast_sender.send_message(&msg).await.unwrap();

    let received = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .unwrap()
        .unwrap();

    match received {
        NetworkMessage::Message { pseudo, content, .. } => {
            assert_eq!(pseudo, "Alice");
            assert_eq!(content, "Test");
        }
        _ => panic!("Expected NetworkMessage::Message"),
    }
}

#[tokio::test]
async fn test_receiver_loop_with_crypto() {
    let receiver = create_test_socket(19009).await;
    let sender = create_test_socket(19010).await;
    let addr: SocketAddr = "127.0.0.1:19009".parse().unwrap();

    let key = test_key();
    let crypto_send = Some(Arc::new(CryptoEngine::new(&key).unwrap()));
    let crypto_recv = Some(Arc::new(CryptoEngine::new(&key).unwrap()));

    let (tx, mut rx) = mpsc::channel::<NetworkMessage>(10);
    let recv_socket = receiver.clone();

    tokio::spawn(async move {
        recever::receiver_loop(recv_socket, crypto_recv, tx).await;
    });

    let multicast_sender = MulticastSender::new(sender, addr, crypto_send);

    let msg = NetworkMessage::Leave {
        pseudo: "Charlie".to_string(),
    };

    multicast_sender.send_message(&msg).await.unwrap();

    let received = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .unwrap()
        .unwrap();

    match received {
        NetworkMessage::Leave { pseudo } => {
            assert_eq!(pseudo, "Charlie");
        }
        _ => panic!("Expected NetworkMessage::Leave"),
    }
}

#[tokio::test]
async fn test_receiver_loop_ignores_invalid_data() {
    let receiver = create_test_socket(19011).await;
    let sender = create_test_socket(19012).await;
    let addr: SocketAddr = "127.0.0.1:19011".parse().unwrap();

    let (tx, mut rx) = mpsc::channel::<NetworkMessage>(10);
    let recv_socket = receiver.clone();

    tokio::spawn(async move {
        recever::receiver_loop(recv_socket, None, tx).await;
    });

    let multicast_sender = MulticastSender::new(sender, addr, None);

    // Send invalid JSON
    multicast_sender.send(b"not json").await.unwrap();

    // Send valid data that is not a NetworkMessage
    multicast_sender.send(b"\"invalid\"").await.unwrap();

    // No message should be received
    let result = tokio::time::timeout(Duration::from_millis(500), rx.recv()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_multiple_messages_received() {
    let receiver = create_test_socket(19013).await;
    let sender = create_test_socket(19014).await;
    let addr: SocketAddr = "127.0.0.1:19013".parse().unwrap();

    let (tx, mut rx) = mpsc::channel::<NetworkMessage>(10);
    let recv_socket = receiver.clone();

    tokio::spawn(async move {
        recever::receiver_loop(recv_socket, None, tx).await;
    });

    let multicast_sender = MulticastSender::new(sender, addr, None);

    for i in 0..5 {
        let msg = NetworkMessage::Message {
            pseudo: "Alice".to_string(),
            content: format!("Message {}", i),
            timestamp: chrono::Utc::now(),
        };
        multicast_sender.send_message(&msg).await.unwrap();
        sleep(Duration::from_millis(50)).await;
    }

    for i in 0..5 {
        let received = tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .unwrap()
            .unwrap();
        match received {
            NetworkMessage::Message { content, .. } => {
                assert_eq!(content, format!("Message {}", i));
            }
            _ => panic!("Expected NetworkMessage::Message"),
        }
    }
}
