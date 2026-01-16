use async_trait::async_trait;
use anyhow::Result;

/// Represents a message from chat
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub username: String,
    pub user_id: Option<String>,
    pub message: String,
    pub channel: String,
}

/// Parsed commands from chat
#[derive(Debug, Clone, PartialEq)]
pub enum ChatCommand {
    /// Answer submission (a, b, c, or d)
    Answer { username: String, user_id: Option<String>, answer: String },
    /// Card vote (card name to use)
    UseCard { username: String, user_id: Option<String>, card_name: String },
    /// Unknown/invalid command
    Unknown,
}

impl ChatMessage {
    /// Parse message into a command if it matches known patterns
    pub fn parse_command(&self) -> ChatCommand {
        let msg = self.message.trim().to_lowercase();

        // Check for single-letter answer (a, b, c, d)
        if msg.len() == 1 && matches!(msg.as_str(), "a" | "b" | "c" | "d") {
            return ChatCommand::Answer {
                username: self.username.clone(),
                user_id: self.user_id.clone(),
                answer: msg,
            };
        }

        // Check for "use <card-name>" pattern
        if let Some(stripped) = msg.strip_prefix("use ") {
            let card_name = stripped.trim().to_string();
            if !card_name.is_empty() {
                return ChatCommand::UseCard {
                    username: self.username.clone(),
                    user_id: self.user_id.clone(),
                    card_name,
                };
            }
        }

        ChatCommand::Unknown
    }
}

/// Trait for chat providers (Twitch, Discord, Mock, etc.)
#[async_trait]
pub trait ChatProvider: Send + Sync {
    /// Connect to the chat service
    async fn connect(&mut self) -> Result<()>;

    /// Join a specific channel/room
    async fn join(&mut self, channel: &str) -> Result<()>;

    /// Receive next message (blocking until message arrives)
    async fn recv_message(&mut self) -> Result<ChatMessage>;

    /// Send a message (optional - not all providers support sending)
    async fn send_message(&mut self, channel: &str, message: &str) -> Result<()> {
        let _ = (channel, message);
        Err(anyhow::anyhow!("Sending messages not supported by this provider"))
    }

    /// Disconnect from the service
    async fn disconnect(&mut self) -> Result<()>;

    /// Get the provider name (for logging)
    fn name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_answer_command() {
        let msg = ChatMessage {
            username: "testuser".to_string(),
            user_id: Some("12345".to_string()),
            message: "a".to_string(),
            channel: "#test".to_string(),
        };

        let cmd = msg.parse_command();
        assert_eq!(cmd, ChatCommand::Answer {
            username: "testuser".to_string(),
            user_id: Some("12345".to_string()),
            answer: "a".to_string(),
        });
    }

    #[test]
    fn test_parse_use_card_command() {
        let msg = ChatMessage {
            username: "testuser".to_string(),
            user_id: Some("12345".to_string()),
            message: "use Yaffa Drone Strike".to_string(),
            channel: "#test".to_string(),
        };

        let cmd = msg.parse_command();
        assert_eq!(cmd, ChatCommand::UseCard {
            username: "testuser".to_string(),
            user_id: Some("12345".to_string()),
            card_name: "yaffa drone strike".to_string(),
        });
    }

    #[test]
    fn test_parse_unknown_command() {
        let msg = ChatMessage {
            username: "testuser".to_string(),
            user_id: None,
            message: "hello world".to_string(),
            channel: "#test".to_string(),
        };

        let cmd = msg.parse_command();
        assert_eq!(cmd, ChatCommand::Unknown);
    }
}
