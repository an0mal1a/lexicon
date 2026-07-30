//! Product domain types for Lexicon. This crate deliberately has no HTTP or LXDB dependency.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChallengeId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GameId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameMode {
    Daily,
    Infinite,
    SemanticPath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameStatus {
    InProgress,
    Won,
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    pub id: ChallengeId,
    pub date: String,
    pub language: String,
    pub mode: GameMode,
    /// This field is domain-only. API response types must never expose it before completion.
    pub target_word: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attempt {
    pub word: String,
    pub score: f32,
    pub distance: Option<u32>,
    pub rank: Option<u32>,
    pub is_exact: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hint {
    pub number: u8,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerStats {
    pub games_played: u32,
    pub games_won: u32,
    pub current_streak: u32,
    pub best_streak: u32,
    pub total_attempts: u32,
}

impl PlayerStats {
    pub fn average_attempts(&self) -> f32 {
        if self.games_won == 0 { 0.0 } else { self.total_attempts as f32 / self.games_won as f32 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSession {
    pub id: GameId,
    pub session_id: SessionId,
    pub challenge: Challenge,
    pub attempts: Vec<Attempt>,
    pub hints_used: u8,
    pub status: GameStatus,
    pub score: Option<u32>,
}
