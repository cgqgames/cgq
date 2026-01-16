use crate::chat::{ChatMessage, ChatProvider};
use async_trait::async_trait;
use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use bevy::prelude::*;

/// Anonymous Twitch IRC chat provider (read-only, no authentication)
pub struct TwitchChatProvider {
    stream: Option<TcpStream>,
    reader: Option<BufReader<tokio::io::ReadHalf<TcpStream>>>,
    writer: Option<tokio::io::WriteHalf<TcpStream>>,
    connected: bool,
}

impl TwitchChatProvider {
    pub fn new() -> Self {
        Self {
            stream: None,
            reader: None,
            writer: None,
            connected: false,
        }
    }

    /// Parse a Twitch IRC PRIVMSG line into a ChatMessage
    fn parse_privmsg(line: &str) -> Option<ChatMessage> {
        // Format: :username!username@username.tmi.twitch.tv PRIVMSG #channel :message
        // Or with tags: @badge-info=;badges=;client-nonce=...;user-id=12345;... :username!username@username.tmi.twitch.tv PRIVMSG #channel :message

        let mut user_id = None;
        let mut username = String::new();
        let mut channel = String::new();
        let mut message = String::new();

        // Check for tags (starts with @)
        let line = if line.starts_with('@') {
            // Extract user-id from tags if present
            if let Some(tags_end) = line.find(" :") {
                let tags = &line[1..tags_end];
                for tag in tags.split(';') {
                    if let Some((key, value)) = tag.split_once('=') {
                        if key == "user-id" && !value.is_empty() {
                            user_id = Some(value.to_string());
                        }
                    }
                }
                &line[tags_end + 1..]
            } else {
                line
            }
        } else {
            line
        };

        // Parse username and message
        // Format: :username!username@username.tmi.twitch.tv PRIVMSG #channel :message
        if let Some(rest) = line.strip_prefix(':') {
            if let Some((user_part, rest)) = rest.split_once('!') {
                username = user_part.to_string();

                if let Some(msg_start) = rest.find(" PRIVMSG ") {
                    let after_privmsg = &rest[msg_start + 9..];
                    if let Some((chan, msg)) = after_privmsg.split_once(" :") {
                        channel = chan.to_string();
                        message = msg.to_string();

                        return Some(ChatMessage {
                            username,
                            user_id,
                            message,
                            channel,
                        });
                    }
                }
            }
        }

        None
    }
}

impl Default for TwitchChatProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ChatProvider for TwitchChatProvider {
    async fn connect(&mut self) -> Result<()> {
        if self.connected {
            return Ok(());
        }

        info!("Connecting to Twitch IRC (anonymous)...");

        let stream = TcpStream::connect("irc.chat.twitch.tv:6667")
            .await
            .context("Failed to connect to Twitch IRC")?;

        let (read_half, write_half) = tokio::io::split(stream);
        self.reader = Some(BufReader::new(read_half));
        self.writer = Some(write_half);

        // Anonymous login (justinfan + random number)
        let nick = format!("justinfan{}", rand::random::<u32>());
        if let Some(writer) = &mut self.writer {
            writer
                .write_all(format!("NICK {}\r\n", nick).as_bytes())
                .await
                .context("Failed to send NICK")?;
        }

        // Request tags capability (to get user-id)
        if let Some(writer) = &mut self.writer {
            writer
                .write_all(b"CAP REQ :twitch.tv/tags\r\n")
                .await
                .context("Failed to request tags capability")?;
        }

        self.connected = true;
        info!("Connected to Twitch IRC as {}", nick);

        Ok(())
    }

    async fn join(&mut self, channel: &str) -> Result<()> {
        if !self.connected {
            return Err(anyhow::anyhow!("Not connected to Twitch IRC"));
        }

        let channel = if channel.starts_with('#') {
            channel.to_string()
        } else {
            format!("#{}", channel)
        };

        info!("Joining Twitch channel: {}", channel);

        if let Some(writer) = &mut self.writer {
            writer
                .write_all(format!("JOIN {}\r\n", channel).as_bytes())
                .await
                .context("Failed to send JOIN")?;
        }

        Ok(())
    }

    async fn recv_message(&mut self) -> Result<ChatMessage> {
        if !self.connected {
            return Err(anyhow::anyhow!("Not connected to Twitch IRC"));
        }

        let reader = self.reader.as_mut().ok_or_else(|| anyhow::anyhow!("No reader available"))?;

        loop {
            let mut line = String::new();
            reader.read_line(&mut line).await.context("Failed to read from IRC")?;

            if line.is_empty() {
                return Err(anyhow::anyhow!("Connection closed"));
            }

            let line = line.trim();

            // Respond to PING
            if line.starts_with("PING") {
                if let Some(writer) = &mut self.writer {
                    writer.write_all(line.replace("PING", "PONG").as_bytes()).await?;
                    writer.write_all(b"\r\n").await?;
                }
                continue;
            }

            // Parse PRIVMSG
            if line.contains(" PRIVMSG ") {
                if let Some(msg) = Self::parse_privmsg(line) {
                    return Ok(msg);
                }
            }

            // Log other messages for debugging
            trace!("IRC: {}", line);
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        if let Some(mut writer) = self.writer.take() {
            let _ = writer.write_all(b"QUIT\r\n").await;
        }
        self.reader = None;
        self.connected = false;
        info!("Disconnected from Twitch IRC");
        Ok(())
    }

    fn name(&self) -> &str {
        "Twitch IRC (Anonymous)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_privmsg_simple() {
        let line = ":testuser!testuser@testuser.tmi.twitch.tv PRIVMSG #testchannel :hello world";
        let msg = TwitchChatProvider::parse_privmsg(line).unwrap();

        assert_eq!(msg.username, "testuser");
        assert_eq!(msg.channel, "#testchannel");
        assert_eq!(msg.message, "hello world");
        assert_eq!(msg.user_id, None);
    }

    #[test]
    fn test_parse_privmsg_with_tags() {
        let line = "@badge-info=;badges=;user-id=12345;display-name=TestUser :testuser!testuser@testuser.tmi.twitch.tv PRIVMSG #testchannel :a";
        let msg = TwitchChatProvider::parse_privmsg(line).unwrap();

        assert_eq!(msg.username, "testuser");
        assert_eq!(msg.channel, "#testchannel");
        assert_eq!(msg.message, "a");
        assert_eq!(msg.user_id, Some("12345".to_string()));
    }

    #[test]
    fn test_parse_privmsg_invalid() {
        let line = "PING :tmi.twitch.tv";
        let msg = TwitchChatProvider::parse_privmsg(line);
        assert!(msg.is_none());
    }
}
