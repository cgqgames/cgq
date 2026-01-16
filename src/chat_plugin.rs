use bevy::prelude::*;
use tokio::sync::mpsc;
use crate::chat::{ChatCommand, ChatProvider};
use crate::twitch::TwitchChatProvider;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Resource that receives chat commands from the chat provider
#[derive(Resource)]
pub struct ChatReceiver {
    pub receiver: mpsc::UnboundedReceiver<ChatCommand>,
}

/// Resource tracking chat answer submissions for consensus
#[derive(Resource, Default)]
pub struct ChatAnswerTracker {
    /// Map of answer (a/b/c/d) -> count of users who submitted it
    pub votes: std::collections::HashMap<String, usize>,
    /// Users who have already voted (to prevent spam)
    pub voted_users: std::collections::HashSet<String>,
}

impl ChatAnswerTracker {
    pub fn reset(&mut self) {
        self.votes.clear();
        self.voted_users.clear();
    }

    pub fn add_vote(&mut self, username: &str, answer: &str) -> bool {
        // Return false if user already voted
        if self.voted_users.contains(username) {
            return false;
        }

        self.voted_users.insert(username.to_string());
        *self.votes.entry(answer.to_string()).or_insert(0) += 1;
        true
    }

    /// Get the most voted answer, if any
    pub fn get_consensus(&self) -> Option<(String, usize)> {
        self.votes
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(answer, count)| (answer.clone(), *count))
    }

    /// Get total number of votes
    pub fn total_votes(&self) -> usize {
        self.votes.values().sum()
    }
}

/// Resource tracking card vote submissions
#[derive(Resource, Default)]
pub struct ChatCardVoteTracker {
    /// Map of card_name -> count of users who voted for it
    pub votes: std::collections::HashMap<String, usize>,
    /// Users who have already voted (to prevent spam)
    pub voted_users: std::collections::HashSet<String>,
}

impl ChatCardVoteTracker {
    pub fn reset(&mut self) {
        self.votes.clear();
        self.voted_users.clear();
    }

    pub fn add_vote(&mut self, username: &str, card_name: &str) -> bool {
        // Return false if user already voted
        if self.voted_users.contains(username) {
            return false;
        }

        self.voted_users.insert(username.to_string());
        *self.votes.entry(card_name.to_string()).or_insert(0) += 1;
        true
    }

    /// Get the most voted card and its vote count
    pub fn get_winner(&self) -> Option<(String, usize)> {
        self.votes
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(card, count)| (card.clone(), *count))
    }

    /// Get total number of votes
    pub fn total_votes(&self) -> usize {
        self.votes.values().sum()
    }
}

/// Spawns an async task that receives messages from the chat provider
pub fn spawn_chat_listener(
    provider: Arc<Mutex<dyn ChatProvider>>,
    sender: mpsc::UnboundedSender<ChatCommand>,
) {
    tokio::spawn(async move {
        let mut provider = provider.lock().await;

        loop {
            match provider.recv_message().await {
                Ok(msg) => {
                    let cmd = msg.parse_command();

                    // Only send non-Unknown commands
                    match cmd {
                        ChatCommand::Answer { .. } | ChatCommand::UseCard { .. } => {
                            if sender.send(cmd).is_err() {
                                error!("Chat receiver channel closed");
                                break;
                            }
                        }
                        ChatCommand::Unknown => {
                            // Ignore unknown commands
                        }
                    }
                }
                Err(e) => {
                    error!("Error receiving chat message: {}", e);
                    break;
                }
            }
        }
    });
}

/// System that processes incoming chat commands
pub fn process_chat_commands(
    mut chat_receiver: ResMut<ChatReceiver>,
    mut answer_tracker: ResMut<ChatAnswerTracker>,
    mut card_tracker: ResMut<ChatCardVoteTracker>,
) {
    // Process all pending commands
    while let Ok(cmd) = chat_receiver.receiver.try_recv() {
        match cmd {
            ChatCommand::Answer { username, answer, .. } => {
                if answer_tracker.add_vote(&username, &answer) {
                    let total = answer_tracker.total_votes();
                    info!("Chat answer: {} voted {}, total votes: {}", username, answer, total);

                    if let Some((consensus, count)) = answer_tracker.get_consensus() {
                        debug!("Current consensus: {} with {} votes", consensus, count);
                    }
                }
            }
            ChatCommand::UseCard { username, card_name, .. } => {
                if card_tracker.add_vote(&username, &card_name) {
                    let total = card_tracker.total_votes();
                    info!("Chat card vote: {} voted for '{}', total votes: {}", username, card_name, total);

                    if let Some((winner, count)) = card_tracker.get_winner() {
                        debug!("Current card leader: '{}' with {} votes", winner, count);
                    }
                }
            }
            ChatCommand::Unknown => {
                // Should not reach here
            }
        }
    }
}

/// Plugin for chat integration
pub struct ChatPlugin {
    pub channel: String,
}

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        let (sender, receiver) = mpsc::unbounded_channel();

        // Create and initialize chat provider
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        let provider: Arc<Mutex<dyn ChatProvider>> = Arc::new(Mutex::new(TwitchChatProvider::new()));

        let channel = self.channel.clone();
        let provider_clone = Arc::clone(&provider);

        // Connect and join channel in blocking context
        runtime.block_on(async {
            let mut p = provider_clone.lock().await;
            if let Err(e) = p.connect().await {
                error!("Failed to connect to chat: {}", e);
                return;
            }
            if let Err(e) = p.join(&channel).await {
                error!("Failed to join channel {}: {}", channel, e);
                return;
            }
            info!("Successfully connected to chat channel: {}", channel);
        });

        // Spawn listener task
        spawn_chat_listener(provider, sender);

        app
            .insert_resource(ChatReceiver { receiver })
            .init_resource::<ChatAnswerTracker>()
            .init_resource::<ChatCardVoteTracker>()
            .add_systems(Update, process_chat_commands);
    }
}
