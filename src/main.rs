use tokio::stream;

async fn handler(stream: stream) {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listeneur = tokio::net::TcpListener::bind("127.0.0.1:7878").await?;

    for stream in listeneur {
        tokio::spawn(handler(stream));
    }
}
