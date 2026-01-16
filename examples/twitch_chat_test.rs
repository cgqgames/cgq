/// Test the Twitch IRC chat provider
/// Run with: cargo run --example twitch_chat_test -- <channel>
///
/// Example: cargo run --example twitch_chat_test -- monokrome

use cgq::chat::{ChatProvider, ChatCommand};
use cgq::twitch::TwitchChatProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <channel>", args[0]);
        eprintln!("Example: {} monokrome", args[0]);
        std::process::exit(1);
    }

    let channel = &args[1];

    println!("=== Twitch Chat Test ===");
    println!("Connecting to channel: {}", channel);
    println!("Type 'a', 'b', 'c', or 'd' to test answer commands");
    println!("Type 'use <card-name>' to test card voting");
    println!("Press Ctrl+C to exit\n");

    let mut provider = TwitchChatProvider::new();

    provider.connect().await?;
    provider.join(channel).await?;

    println!("Connected! Listening for messages...\n");

    loop {
        match provider.recv_message().await {
            Ok(msg) => {
                let cmd = msg.parse_command();
                match cmd {
                    ChatCommand::Answer { username, user_id, answer } => {
                        println!("[ANSWER] {} ({:?}): {}", username, user_id, answer);
                    }
                    ChatCommand::UseCard { username, user_id, card_name } => {
                        println!("[CARD] {} ({:?}): use '{}'", username, user_id, card_name);
                    }
                    ChatCommand::Unknown => {
                        println!("[CHAT] {}: {}", msg.username, msg.message);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                break;
            }
        }
    }

    provider.disconnect().await?;
    Ok(())
}
