use bevy::prelude::*;
use tokio::sync::mpsc;
use crate::chat::{ChatCommand, ChatProvider};
use crate::twitch::TwitchChatProvider;
use crate::components::{Question, ActiveQuestion};
use crate::resources::{QuizState, Score};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Resource that receives chat commands from the chat provider
#[derive(Resource)]
pub struct ChatReceiver {
    pub receiver: mpsc::UnboundedReceiver<ChatCommand>,
}

/// Configuration for chat consensus
#[derive(Resource)]
pub struct ChatConsensusConfig {
    /// Minimum number of votes required to trigger consensus answer
    pub answer_threshold: usize,
    /// Minimum number of votes required to activate a card
    pub card_threshold: usize,
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
    pub answer_threshold: usize,
    pub card_threshold: usize,
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
            .insert_resource(ChatConsensusConfig {
                answer_threshold: self.answer_threshold,
                card_threshold: self.card_threshold,
            })
            .init_resource::<ChatAnswerTracker>()
            .init_resource::<ChatCardVoteTracker>()
            .add_systems(Update, (
                process_chat_commands,
                check_answer_consensus,
                reset_votes_on_question_change,
            ));
    }
}

/// System that checks if consensus is reached and submits the answer
pub fn check_answer_consensus(
    config: Res<ChatConsensusConfig>,
    mut answer_tracker: ResMut<ChatAnswerTracker>,
    mut quiz_state: ResMut<QuizState>,
    mut score: ResMut<Score>,
    questions: Query<&Question, With<ActiveQuestion>>,
) {
    if !quiz_state.game_started || quiz_state.paused || quiz_state.game_complete {
        return;
    }

    // Check if we have enough votes
    let total_votes = answer_tracker.total_votes();
    if total_votes < config.answer_threshold {
        return;
    }

    // Get consensus answer
    if let Some((answer, count)) = answer_tracker.get_consensus() {
        info!("Chat consensus reached: {} with {} votes (threshold: {})",
              answer, count, config.answer_threshold);

        // Submit the answer
        if let Ok(question) = questions.get_single() {
            let correct = question.is_correct(&answer);

            if correct {
                score.current += question.points;
                score.correct_answers += 1;
                info!("✅ Chat answered correctly! +{} points. Score: {}",
                      question.points, score.current);
            } else {
                info!("❌ Chat answered incorrectly! Correct answer: {:?}",
                      question.correct_answer().map(|o| &o.id));
            }

            score.total_answered += 1;

            // Auto-advance to next question
            quiz_state.current_question_index += 1;

            if quiz_state.current_question_index >= quiz_state.total_questions {
                quiz_state.game_complete = true;
                info!("🏁 Quiz complete! Final score: {} / {}",
                      score.current, score.passing_grade);
            } else {
                info!("Moving to question {}", quiz_state.current_question_index + 1);
            }

            // Reset tracker for next question
            answer_tracker.reset();
        }
    }
}

/// System that resets vote trackers when the question changes
pub fn reset_votes_on_question_change(
    quiz_state: Res<QuizState>,
    mut answer_tracker: ResMut<ChatAnswerTracker>,
    mut card_tracker: ResMut<ChatCardVoteTracker>,
) {
    // Reset when question index changes
    if quiz_state.is_changed() && quiz_state.game_started {
        info!("Question changed, resetting chat votes");
        answer_tracker.reset();
        card_tracker.reset();
    }
}
