//! LXDB-backed semantic rules for Lexicon.
//!
//! The engine reads only LXDB's public query API. It never knows about HTTP,
//! browser state, or persistence.

use std::{fmt, sync::Arc};

use lexicon_core::{
    Attempt, Challenge, ChallengeId, GameId, GameMode, GameSession, GameStatus, Hint, SessionId,
};
use lxdb::{BinaryDataset, BinaryDatasetExt, core::ids::TokenId};

const MAX_WORD_BYTES: usize = 64;
const MAX_PATH_DEPTH: u32 = 4;
const HOP_DECAY: f32 = 0.82;
const INVERSE_PENALTY: f32 = 0.85;

#[derive(Debug, Clone, PartialEq)]
pub struct SimilarityResult {
    pub normalized_score: f32,
    pub distance: Option<u32>,
    pub direct: bool,
    pub path: Vec<TokenId>,
}

#[derive(Debug)]
pub enum GameError {
    UnknownWord,
    RepeatedWord,
    GameFinished,
    InvalidWord,
    Dataset(String),
}

impl fmt::Display for GameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownWord => write!(formatter, "word is not present in the dataset"),
            Self::RepeatedWord => write!(formatter, "word has already been attempted"),
            Self::GameFinished => write!(formatter, "game is no longer active"),
            Self::InvalidWord => write!(formatter, "word is invalid"),
            Self::Dataset(message) => write!(formatter, "dataset query failed: {message}"),
        }
    }
}

impl std::error::Error for GameError {}

/// Deterministic game rules backed by one immutable LXDB dataset.
#[derive(Debug, Clone)]
pub struct GameEngine {
    dataset: Arc<BinaryDataset>,
}

impl GameEngine {
    pub fn new(dataset: Arc<BinaryDataset>) -> Self {
        Self { dataset }
    }

    pub fn dataset(&self) -> &BinaryDataset {
        &self.dataset
    }

    pub fn normalize_word(&self, value: &str) -> Result<String, GameError> {
        let word = value.trim().to_lowercase();
        if word.is_empty() || word.len() > MAX_WORD_BYTES || word.contains(['\r', '\n', '\0']) {
            return Err(GameError::InvalidWord);
        }
        Ok(word)
    }

    pub fn daily_challenge(&self, language: &str, date: &str) -> Result<Challenge, GameError> {
        let mut eligible = Vec::new();
        for token in self.dataset.resolved_tokens() {
            let token = token.map_err(dataset_error)?;
            if self.dataset.query().outgoing(token.id()).map_err(dataset_error)?.len() > 0 {
                eligible.push(token.text().to_owned());
            }
        }

        if eligible.is_empty() {
            return Err(GameError::Dataset("no playable tokens in dataset".to_owned()));
        }

        let seed = stable_hash(&format!("{language}:{date}"));
        let target_word = eligible[seed as usize % eligible.len()].clone();
        Ok(Challenge {
            id: ChallengeId(format!("daily-{language}-{date}")),
            date: date.to_owned(),
            language: language.to_owned(),
            mode: GameMode::Daily,
            target_word,
        })
    }

    pub fn start_daily(
        &self,
        language: &str,
        date: &str,
        session_id: SessionId,
        game_id: GameId,
    ) -> Result<GameSession, GameError> {
        Ok(GameSession {
            id: game_id,
            session_id,
            challenge: self.daily_challenge(language, date)?,
            attempts: Vec::new(),
            hints_used: 0,
            status: GameStatus::InProgress,
            score: None,
        })
    }

    pub fn attempt(
        &self,
        game: &mut GameSession,
        input: &str,
        now: String,
    ) -> Result<Attempt, GameError> {
        if game.status != GameStatus::InProgress {
            return Err(GameError::GameFinished);
        }
        let word = self.normalize_word(input)?;
        if game.attempts.iter().any(|attempt| attempt.word == word) {
            return Err(GameError::RepeatedWord);
        }

        let similarity = self.similarity(&word, &game.challenge.target_word)?;
        let attempt = Attempt {
            is_exact: similarity.normalized_score == 1.0,
            word,
            score: similarity.normalized_score,
            distance: similarity.distance,
            rank: self.rank_for(&similarity, &game.challenge.target_word)?,
            created_at: now,
        };
        if attempt.is_exact {
            game.status = GameStatus::Won;
            game.score = Some(score(game.attempts.len() as u32 + 1, game.hints_used));
        }
        game.attempts.push(attempt.clone());
        Ok(attempt)
    }

    pub fn hint(&self, game: &mut GameSession) -> Result<Hint, GameError> {
        if game.status != GameStatus::InProgress {
            return Err(GameError::GameFinished);
        }
        let number = game.hints_used.saturating_add(1);
        let text = match number {
            1 => format!("La palabra tiene {} letras.", game.challenge.target_word.chars().count()),
            2 => game
                .challenge
                .target_word
                .chars()
                .next()
                .map(|letter| format!("Empieza por «{letter}»."))
                .unwrap_or_else(|| "No hay una pista disponible.".to_owned()),
            _ => {
                "Busca una palabra con conexiones semánticas directas en el diccionario.".to_owned()
            }
        };
        game.hints_used = number;
        Ok(Hint { number, text })
    }

    pub fn abandon(&self, game: &mut GameSession) -> Result<(), GameError> {
        if game.status != GameStatus::InProgress {
            return Err(GameError::GameFinished);
        }
        game.status = GameStatus::Abandoned;
        Ok(())
    }

    pub fn similarity(&self, input: &str, target: &str) -> Result<SimilarityResult, GameError> {
        let input = self.normalize_word(input)?;
        let target = self.normalize_word(target)?;
        let query = self.dataset.query();
        let source =
            query.token_by_text(&input).map_err(dataset_error)?.ok_or(GameError::UnknownWord)?;
        let target =
            query.token_by_text(&target).map_err(dataset_error)?.ok_or(GameError::UnknownWord)?;

        if source.id() == target.id() {
            return Ok(SimilarityResult {
                normalized_score: 1.0,
                distance: Some(0),
                direct: true,
                path: vec![source.id()],
            });
        }

        let mut best = SimilarityResult {
            normalized_score: 0.0,
            distance: None,
            direct: false,
            path: Vec::new(),
        };
        let mut path = vec![source.id()];
        self.search_paths(source.id(), target.id(), MAX_PATH_DEPTH, 1.0, &mut path, &mut best)?;

        if best.distance.is_none() {
            for relation in self.dataset.relations() {
                let relation = relation.map_err(dataset_error)?;
                if relation.source() == target.id().value()
                    && relation.target() == source.id().value()
                {
                    best = SimilarityResult {
                        normalized_score: (relation.weight() * INVERSE_PENALTY).clamp(0.0, 1.0),
                        distance: Some(1),
                        direct: true,
                        path: vec![source.id(), target.id()],
                    };
                    break;
                }
            }
        }
        Ok(best)
    }

    fn search_paths(
        &self,
        current: TokenId,
        target: TokenId,
        remaining: u32,
        accumulated: f32,
        path: &mut Vec<TokenId>,
        best: &mut SimilarityResult,
    ) -> Result<(), GameError> {
        if remaining == 0 {
            return Ok(());
        }
        let relations = self.dataset.query().resolved_outgoing(current).map_err(dataset_error)?;
        for relation in relations {
            let relation = relation.map_err(dataset_error)?;
            let next = relation.target().id();
            if path.contains(&next) {
                continue;
            }
            let next_score =
                accumulated * relation.weight() * if path.len() > 1 { HOP_DECAY } else { 1.0 };
            path.push(next);
            if next == target {
                let distance = (path.len() - 1) as u32;
                if next_score > best.normalized_score {
                    *best = SimilarityResult {
                        normalized_score: next_score.clamp(0.0, 1.0),
                        distance: Some(distance),
                        direct: distance == 1,
                        path: path.clone(),
                    };
                }
            } else {
                self.search_paths(next, target, remaining - 1, next_score, path, best)?;
            }
            path.pop();
        }
        Ok(())
    }

    fn rank_for(
        &self,
        similarity: &SimilarityResult,
        target: &str,
    ) -> Result<Option<u32>, GameError> {
        if similarity.distance != Some(1) {
            return Ok(None);
        }
        let target = self
            .dataset
            .query()
            .token_by_text(target)
            .map_err(dataset_error)?
            .ok_or(GameError::UnknownWord)?;
        let mut higher = 0_u32;
        for relation in self.dataset.relations() {
            let relation = relation.map_err(dataset_error)?;
            if relation.target() == target.id().value()
                && relation.weight() > similarity.normalized_score
            {
                higher = higher.saturating_add(1);
            }
        }
        Ok(Some(higher + 1))
    }
}

pub fn score(attempts: u32, hints_used: u8) -> u32 {
    1_000_u32
        .saturating_sub(attempts.saturating_sub(1).saturating_mul(55))
        .saturating_sub(u32::from(hints_used).saturating_mul(120))
        .max(100)
}

fn stable_hash(value: &str) -> u64 {
    value.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn dataset_error(error: impl fmt::Display) -> GameError {
    GameError::Dataset(error.to_string())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::Arc,
        time::{SystemTime, UNIX_EPOCH},
    };

    use lexicon_core::{GameId, GameStatus, SessionId};
    use lxdb::DatasetReader;
    use lxdb_compiler::{builder::Builder, compiler::Compiler};

    use super::{GameEngine, GameError, score};

    fn engine() -> GameEngine {
        let directory = std::env::temp_dir().join(format!(
            "lexicon-game-test-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).expect("valid system time").as_nanos()
        ));
        fs::create_dir_all(&directory).expect("temporary directory should be created");
        let source = directory.join("fixture.lx");
        let output = directory.join("fixture.lxdb");
        fs::write(
            &source,
            "coche -> automovil : 0.97\nautomovil -> coche : 0.97\nviaje -> coche : 0.75\n",
        )
        .expect("fixture should be written");
        Compiler::new()
            .compile(
                Builder::new()
                    .input(source.to_string_lossy().into_owned())
                    .output(output.to_string_lossy().into_owned())
                    .build(),
            )
            .expect("fixture should compile");
        GameEngine::new(Arc::new(DatasetReader::new().open(output).expect("fixture should open")))
    }

    #[test]
    fn scores_exact_direct_and_indirect_words_from_lxdb() {
        let engine = engine();
        let exact = engine.similarity("automovil", "automovil").expect("exact word should score");
        let direct = engine.similarity("coche", "automovil").expect("direct word should score");
        let indirect = engine.similarity("viaje", "automovil").expect("indirect word should score");
        assert_eq!(exact.normalized_score, 1.0);
        assert_eq!(direct.distance, Some(1));
        assert!(direct.normalized_score > indirect.normalized_score);
        assert_eq!(indirect.distance, Some(2));
    }

    #[test]
    fn rejects_unknown_and_repeated_attempts_then_records_victory() {
        let engine = engine();
        let mut game = engine
            .start_daily(
                "es",
                "2026-07-30",
                SessionId("session".to_owned()),
                GameId("game".to_owned()),
            )
            .expect("daily game should start");
        let target = game.challenge.target_word.clone();
        assert!(matches!(
            engine.attempt(&mut game, "ausente", "0".to_owned()),
            Err(GameError::UnknownWord)
        ));
        engine.attempt(&mut game, "coche", "1".to_owned()).expect("first attempt should work");
        assert!(matches!(
            engine.attempt(&mut game, "coche", "2".to_owned()),
            Err(GameError::RepeatedWord)
        ));
        engine.attempt(&mut game, &target, "3".to_owned()).expect("target should win");
        assert_eq!(game.status, GameStatus::Won);
        assert!(game.score.is_some());
    }

    #[test]
    fn hints_abandon_and_daily_selection_are_deterministic() {
        let engine = engine();
        let first =
            engine.daily_challenge("es", "2026-07-30").expect("challenge should be selected");
        let second =
            engine.daily_challenge("es", "2026-07-30").expect("challenge should be selected");
        assert_eq!(first.target_word, second.target_word);
        let mut game = engine
            .start_daily(
                "es",
                "2026-07-30",
                SessionId("session".to_owned()),
                GameId("game".to_owned()),
            )
            .expect("daily game should start");
        assert!(engine.hint(&mut game).expect("hint should be returned").text.contains("letras"));
        engine.abandon(&mut game).expect("active game can be abandoned");
        assert_eq!(game.status, GameStatus::Abandoned);
        assert_eq!(score(1, 0), 1_000);
        assert!(score(9, 2) < 1_000);
    }
}
