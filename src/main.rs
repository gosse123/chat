use std::io::Write;

use chat::app::chat::Chat;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let pseudo = if args.len() > 1 {
        args[1].clone()
    } else {
        print!("Entrez votre pseudo: ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        input.trim().to_string()
    };

    if pseudo.is_empty() {
        eprintln!("Le pseudo ne peut pas être vide.");
        return Ok(());
    }

    let key = args
        .iter()
        .position(|a| a == "--key")
        .and_then(|i| args.get(i + 1))
        .cloned();

    let mut chat = Chat::new(pseudo, key).await?;
    chat.run().await?;

    Ok(())
}
