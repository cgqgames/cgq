use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::{mpsc, Mutex};

use crate::chat::{ChatCommand, ChatProvider};
use crate::components::{ActiveQuestion, Question};
use crate::deploy::AnswerSubmittedEvent;
use crate::modes::QuizMode;
use crate::players::{stable_id, upsert_player, AnswerRecord, Player, PlayerRegistry};
use crate::resources::{QuizState, Score};
use crate::twitch::TwitchChatProvider;

#[derive(Resource)]
pub struct ChatReceiver {
    pub receiver: mpsc::UnboundedReceiver<ChatCommand>,
}

#[derive(Resource)]
pub struct ChatConsensusConfig {
    /// Minimum votes required to resolve a consensus answer.
    pub answer_threshold: usize,
    /// Minimum votes required to deploy a card.
    pub card_threshold: usize,
}

/// Per-question answer votes, keyed by stable player id. Enough to
/// reconstruct both the consensus and the per-player attribution.
#[derive(Resource, Default)]
pub struct ChatAnswerTracker {
    pub votes: HashMap<String, PlayerVote>,
}

#[derive(Clone, Debug)]
pub struct PlayerVote {
    pub username: String,
    pub answer: String,
}

impl ChatAnswerTracker {
    pub fn reset(&mut self) {
        self.votes.clear();
    }

    /// Record a vote for this player. Returns false if the player already
    /// voted (spam prevention).
    pub fn add_vote(&mut self, player_id: &str, username: &str, answer: &str) -> bool {
        if self.votes.contains_key(player_id) {
            return false;
        }
        self.votes.insert(
            player_id.to_string(),
            PlayerVote {
                username: username.to_string(),
                answer: answer.to_string(),
            },
        );
        true
    }

    pub fn total_votes(&self) -> usize {
        self.votes.len()
    }

    pub fn get_consensus(&self) -> Option<(String, usize)> {
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for vote in self.votes.values() {
            *counts.entry(vote.answer.as_str()).or_insert(0) += 1;
        }
        counts
            .into_iter()
            .max_by_key(|(_, c)| *c)
            .map(|(a, c)| (a.to_string(), c))
    }
}

/// Per-question card-deploy votes, keyed by stable player id.
#[derive(Resource, Default)]
pub struct ChatCardVoteTracker {
    pub votes: HashMap<String, PlayerCardVote>,
}

#[derive(Clone, Debug)]
pub struct PlayerCardVote {
    pub username: String,
    pub card_name: String,
}

impl ChatCardVoteTracker {
    pub fn reset(&mut self) {
        self.votes.clear();
    }

    pub fn add_vote(&mut self, player_id: &str, username: &str, card_name: &str) -> bool {
        if self.votes.contains_key(player_id) {
            return false;
        }
        self.votes.insert(
            player_id.to_string(),
            PlayerCardVote {
                username: username.to_string(),
                card_name: card_name.to_string(),
            },
        );
        true
    }

    pub fn total_votes(&self) -> usize {
        self.votes.len()
    }

    /// Count how many players voted for a given (case-insensitive) card name.
    pub fn count_for(&self, card_name: &str) -> usize {
        self.votes
            .values()
            .filter(|v| v.card_name.eq_ignore_ascii_case(card_name))
            .count()
    }
}

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
                    match cmd {
                        ChatCommand::Answer { .. } | ChatCommand::UseCard { .. } => {
                            if sender.send(cmd).is_err() {
                                error!("Chat receiver channel closed");
                                break;
                            }
                        }
                        ChatCommand::Unknown => {}
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

pub fn process_chat_commands(
    mut commands: Commands,
    mut chat_receiver: ResMut<ChatReceiver>,
    mut answer_tracker: ResMut<ChatAnswerTracker>,
    mut card_tracker: ResMut<ChatCardVoteTracker>,
    mut registry: ResMut<PlayerRegistry>,
    mut players: Query<&mut Player>,
) {
    let now = Instant::now();
    while let Ok(cmd) = chat_receiver.receiver.try_recv() {
        match cmd {
            ChatCommand::Answer { username, user_id, answer } => {
                let pid = stable_id(user_id.as_deref(), &username);
                let entity = upsert_player(&mut commands, &mut registry, &mut players, &pid, &username);
                if is_timed_out(&players, entity, now) {
                    continue;
                }
                if answer_tracker.add_vote(&pid, &username, &answer) {
                    info!(
                        "Chat answer: {} voted {} (total: {})",
                        username,
                        answer,
                        answer_tracker.total_votes()
                    );
                }
            }
            ChatCommand::UseCard { username, user_id, card_name } => {
                let pid = stable_id(user_id.as_deref(), &username);
                let entity = upsert_player(&mut commands, &mut registry, &mut players, &pid, &username);
                if is_timed_out(&players, entity, now) {
                    continue;
                }
                if card_tracker.add_vote(&pid, &username, &card_name) {
                    info!(
                        "Chat card vote: {} voted '{}' (total: {})",
                        username,
                        card_name,
                        card_tracker.total_votes()
                    );
                }
            }
            ChatCommand::Unknown => {}
        }
    }
}

fn is_timed_out(players: &Query<&mut Player>, entity: Entity, now: Instant) -> bool {
    players
        .get(entity)
        .ok()
        .and_then(|p| p.timeout_until)
        .is_some_and(|until| until > now)
}

pub struct ChatPlugin {
    pub channel: String,
    pub answer_threshold: usize,
    pub card_threshold: usize,
}

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        let (sender, receiver) = mpsc::unbounded_channel();

        let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        let provider: Arc<Mutex<dyn ChatProvider>> = Arc::new(Mutex::new(TwitchChatProvider::new()));

        let channel = self.channel.clone();
        let provider_clone = Arc::clone(&provider);
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

        spawn_chat_listener(provider, sender);

        app.insert_resource(ChatReceiver { receiver })
            .insert_resource(ChatConsensusConfig {
                answer_threshold: self.answer_threshold,
                card_threshold: self.card_threshold,
            })
            .init_resource::<ChatAnswerTracker>()
            .init_resource::<ChatCardVoteTracker>()
            .add_systems(
                Update,
                (
                    process_chat_commands,
                    check_answer_consensus,
                    check_card_consensus,
                    reset_votes_on_question_change,
                )
                    .run_if(in_state(QuizMode::Active)),
            );
    }
}

/// When chat consensus reaches threshold, resolve the question. Team score
/// advances on consensus correctness; per-player answer histories record
/// individual correctness regardless of what the team voted.
pub fn check_answer_consensus(
    config: Res<ChatConsensusConfig>,
    mut answer_tracker: ResMut<ChatAnswerTracker>,
    mut quiz_state: ResMut<QuizState>,
    mut score: ResMut<Score>,
    mut answer_events: EventWriter<AnswerSubmittedEvent>,
    questions: Query<&Question, With<ActiveQuestion>>,
    registry: Res<PlayerRegistry>,
    mut players: Query<&mut Player>,
) {
    if quiz_state.paused {
        return;
    }
    if answer_tracker.total_votes() < config.answer_threshold {
        return;
    }

    let Some((consensus_answer, count)) = answer_tracker.get_consensus() else { return };
    info!(
        "Chat consensus reached: {} with {} votes (threshold: {})",
        consensus_answer, count, config.answer_threshold
    );

    let Ok(question) = questions.get_single() else { return };
    let now = Instant::now();
    let question_id = question.id.clone();

    let mut correct_voters: Vec<String> = Vec::new();
    let mut wrong_voters: Vec<String> = Vec::new();
    for (pid, vote) in &answer_tracker.votes {
        let was_correct = question.is_correct(&vote.answer);
        if was_correct {
            correct_voters.push(pid.clone());
        } else {
            wrong_voters.push(pid.clone());
        }
        if let Some(entity) = registry.get(pid) {
            if let Ok(mut player) = players.get_mut(entity) {
                player.answer_history.push(AnswerRecord {
                    question_id: question_id.clone(),
                    vote: vote.answer.clone(),
                    was_correct,
                    at: now,
                });
            }
        }
    }

    let consensus_correct = question.is_correct(&consensus_answer);
    if consensus_correct {
        score.current += question.points;
        score.correct_answers += 1;
        info!(
            "✅ Team answered correctly! +{} points. Score: {}",
            question.points, score.current
        );
    } else {
        info!(
            "❌ Team answered incorrectly! Correct answer: {:?}",
            question.correct_answer().map(|o| &o.id)
        );
    }

    answer_events.send(AnswerSubmittedEvent {
        correct: consensus_correct,
        question_id,
        correct_voters,
        wrong_voters,
    });

    score.total_answered += 1;
    quiz_state.current_question_index += 1;
    if quiz_state.current_question_index < quiz_state.total_questions {
        info!("Moving to question {}", quiz_state.current_question_index + 1);
    }
    answer_tracker.reset();
}

/// Deploy cards that have enough votes, consulting per-type vote-req
/// modifiers. Ties resolve by whichever card is checked first.
pub fn check_card_consensus(
    mut card_tracker: ResMut<ChatCardVoteTracker>,
    mut card_manager: ResMut<crate::resources::CardManager>,
    quiz_state: Res<QuizState>,
) {
    if quiz_state.paused {
        return;
    }

    let ready: Vec<(String, String)> = card_manager
        .available_cards
        .iter()
        .filter_map(|c| {
            if card_manager.deployed_card_ids.contains(&c.id) {
                return None;
            }
            let effective = effective_vote_requirement(c, &card_manager);
            (card_tracker.count_for(&c.name) >= effective)
                .then(|| (c.id.clone(), c.name.clone()))
        })
        .collect();

    for (id, name) in ready {
        info!("🎴 Chat activated card: {}", name);
        card_manager.deployed_card_ids.push(id);
        card_tracker
            .votes
            .retain(|_, v| !v.card_name.eq_ignore_ascii_case(&name));
    }
}

fn effective_vote_requirement(
    card: &crate::resources::CardDefinition,
    cm: &crate::resources::CardManager,
) -> usize {
    let type_key = card_type_key(&card.card_type);
    let modifier = cm.vote_req_modifiers.get(&type_key).copied().unwrap_or(0)
        + cm.vote_req_modifiers.get("*").copied().unwrap_or(0);
    (card.vote_requirement as i32 + modifier).max(1) as usize
}

fn card_type_key(card_type: &crate::components::CardType) -> String {
    use crate::components::CardType::*;
    match card_type {
        Resistance => "resistance",
        Palestinian => "palestinian",
        Politics => "politics",
        Negative => "negative",
        IDF => "idf",
        Hasbara => "hasbara",
        Ceasefire => "ceasefire",
        Other => "other",
    }
    .to_string()
}

pub fn reset_votes_on_question_change(
    quiz_state: Res<QuizState>,
    mut answer_tracker: ResMut<ChatAnswerTracker>,
    mut card_tracker: ResMut<ChatCardVoteTracker>,
) {
    if quiz_state.is_changed() {
        info!("Question changed, resetting chat votes");
        answer_tracker.reset();
        card_tracker.reset();
    }
}
