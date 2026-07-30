use std::{
    collections::HashMap,
    env, fs,
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use lexicon_core::{GameId, GameSession, GameStatus, PlayerStats, SessionId};
use lexicon_game::{GameEngine, GameError};
use lxdb::DatasetReader;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct AppState {
    engine: GameEngine,
    repository: Arc<Mutex<Repository>>,
    date: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PersistedStore {
    games: HashMap<String, GameSession>,
    stats: HashMap<String, PlayerStats>,
}

#[derive(Debug)]
struct Repository {
    path: PathBuf,
    store: PersistedStore,
    next_id: u64,
}

impl Repository {
    fn open(path: PathBuf) -> Self {
        let store = fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        Self { path, store, next_id: 0 }
    }

    fn new_id(&mut self, prefix: &str) -> String {
        self.next_id = self.next_id.wrapping_add(1);
        format!("{prefix}-{:x}-{:x}", unix_seconds(), self.next_id)
    }

    fn persist(&self) -> Result<(), ApiError> {
        let Some(parent) = self.path.parent() else {
            return Err(ApiError::Internal);
        };
        fs::create_dir_all(parent).map_err(|_| ApiError::Internal)?;
        let temporary = self.path.with_extension("tmp");
        let bytes = serde_json::to_vec(&self.store).map_err(|_| ApiError::Internal)?;
        fs::write(&temporary, bytes).map_err(|_| ApiError::Internal)?;
        if self.path.exists() {
            fs::remove_file(&self.path).map_err(|_| ApiError::Internal)?;
        }
        fs::rename(temporary, &self.path).map_err(|_| ApiError::Internal)
    }
}

#[derive(Debug)]
enum ApiError {
    BadRequest(&'static str),
    NotFound,
    Forbidden,
    Conflict(&'static str),
    DatasetUnavailable,
    Internal,
}

impl From<GameError> for ApiError {
    fn from(error: GameError) -> Self {
        match error {
            GameError::UnknownWord => {
                Self::BadRequest("La palabra no existe en el diccionario activo.")
            }
            GameError::InvalidWord => Self::BadRequest("La palabra no es válida."),
            GameError::RepeatedWord => Self::Conflict("Ya probaste esa palabra."),
            GameError::GameFinished => Self::Conflict("La partida ya está cerrada."),
            GameError::Dataset(_) => Self::DatasetUnavailable,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            Self::NotFound => (StatusCode::NOT_FOUND, "Partida no encontrada."),
            Self::Forbidden => {
                (StatusCode::FORBIDDEN, "La sesión no puede acceder a esta partida.")
            }
            Self::Conflict(message) => (StatusCode::CONFLICT, message),
            Self::DatasetUnavailable => {
                (StatusCode::SERVICE_UNAVAILABLE, "El dataset LXDB no está disponible.")
            }
            Self::Internal => {
                (StatusCode::INTERNAL_SERVER_ERROR, "No se pudo completar la operación.")
            }
        };
        (status, Json(ErrorResponse { error: message })).into_response()
    }
}

#[derive(Serialize)]
struct ErrorResponse<'a> {
    error: &'a str,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    dataset_loaded: bool,
    tokens: usize,
    relations: usize,
}

#[derive(Serialize)]
struct DailyResponse {
    challenge_id: String,
    date: String,
    language: String,
    mode: &'static str,
    status: &'static str,
    completed: bool,
}

#[derive(Serialize)]
struct PublicGame {
    game_id: String,
    challenge_id: String,
    date: String,
    language: String,
    mode: &'static str,
    status: GameStatus,
    attempts: Vec<lexicon_core::Attempt>,
    hints_used: u8,
    score: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_word: Option<String>,
}

#[derive(Deserialize)]
struct StartRequest {
    language: Option<String>,
}

#[derive(Deserialize)]
struct AttemptRequest {
    word: String,
}

#[derive(Serialize)]
struct AttemptResponse {
    attempt: lexicon_core::Attempt,
    status: GameStatus,
    attempt_count: usize,
    score: Option<u32>,
}

#[derive(Serialize)]
struct HintResponse {
    hint: lexicon_core::Hint,
    hints_used: u8,
}

#[derive(Serialize)]
struct StatsResponse {
    #[serde(flatten)]
    stats: PlayerStats,
    average_attempts: f32,
}

#[tokio::main]
async fn main() {
    let dataset_path = env::var("LEXICON_DATASET")
        .unwrap_or_else(|_| "datasets/generated/es-dev.lxdb".to_owned());
    let dataset = match DatasetReader::new().open(&dataset_path) {
        Ok(dataset) => Arc::new(dataset),
        Err(error) => {
            eprintln!("error: cannot load LXDB dataset {dataset_path}: {error}");
            eprintln!(
                "hint: cargo run -p lxdb-cli -- compile datasets/fixtures/es-dev.lx -o datasets/generated/es-dev.lxdb"
            );
            std::process::exit(1);
        }
    };
    let state = AppState {
        engine: GameEngine::new(dataset),
        repository: Arc::new(Mutex::new(Repository::open(PathBuf::from(
            ".lexicon/sessions.json",
        )))),
        date: env::var("LEXICON_DATE").unwrap_or_else(|_| today_utc()),
    };
    let address = env::var("LEXICON_ADDR").unwrap_or_else(|_| "127.0.0.1:3001".to_owned());
    let address: SocketAddr = match address.parse() {
        Ok(address) => address,
        Err(_) => {
            eprintln!("error: LEXICON_ADDR must be a valid socket address");
            std::process::exit(1);
        }
    };
    println!("Lexicon API listening on http://{address}");
    let listener = match tokio::net::TcpListener::bind(address).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("error: cannot bind API listener: {error}");
            std::process::exit(1);
        }
    };
    if let Err(error) = axum::serve(listener, app(state)).await {
        eprintln!("error: server stopped: {error}");
    }
}

fn app(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health).options(preflight))
        .route("/api/game/daily", get(daily).options(preflight))
        .route("/api/game/daily/start", post(start_daily).options(preflight))
        .route("/api/game/{game_id}/attempt", post(attempt).options(preflight))
        .route("/api/game/{game_id}/hint", post(hint).options(preflight))
        .route("/api/game/{game_id}/abandon", post(abandon).options(preflight))
        .route("/api/game/{game_id}", get(game).options(preflight))
        .route("/api/stats", get(stats).options(preflight))
        .route("/api/modes", get(modes).options(preflight))
        .with_state(state)
}

fn cors_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    let origin = env::var("LEXICON_CORS_ORIGIN")
        .ok()
        .and_then(|value| HeaderValue::from_str(&value).ok())
        .unwrap_or_else(|| HeaderValue::from_static("http://localhost:3000"));
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
    headers.insert(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, HeaderValue::from_static("true"));
    headers.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static("content-type"));
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("GET, POST, OPTIONS"),
    );
    headers
}

async fn preflight() -> (StatusCode, HeaderMap) {
    (StatusCode::NO_CONTENT, cors_headers())
}

async fn health(State(state): State<AppState>) -> (HeaderMap, Json<HealthResponse>) {
    let dataset = state.engine.dataset();
    (
        cors_headers(),
        Json(HealthResponse {
            status: "ok",
            dataset_loaded: true,
            tokens: dataset.token_count(),
            relations: dataset.relation_count(),
        }),
    )
}

async fn daily(
    State(state): State<AppState>,
) -> Result<(HeaderMap, Json<DailyResponse>), ApiError> {
    let challenge = state.engine.daily_challenge("es", &state.date)?;
    Ok((
        cors_headers(),
        Json(DailyResponse {
            challenge_id: challenge.id.0,
            date: challenge.date,
            language: challenge.language,
            mode: "daily",
            status: "available",
            completed: false,
        }),
    ))
}

async fn start_daily(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<StartRequest>,
) -> Result<(HeaderMap, Json<PublicGame>), ApiError> {
    let language = request.language.unwrap_or_else(|| "es".to_lowercase());
    if language != "es" {
        return Err(ApiError::BadRequest("El dataset cargado sólo contiene español."));
    }
    let mut repository = state.repository.lock().map_err(|_| ApiError::Internal)?;
    let session_id =
        session_from_headers(&headers).unwrap_or_else(|| SessionId(repository.new_id("session")));
    let game_id = GameId(repository.new_id("game"));
    let game =
        state.engine.start_daily(&language, &state.date, session_id.clone(), game_id.clone())?;
    repository.store.games.insert(game_id.0.clone(), game.clone());
    repository.store.stats.entry(session_id.0.clone()).or_default();
    repository.persist()?;
    let mut response_headers = cors_headers();
    response_headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&format!(
            "lexicon_session={}; HttpOnly; SameSite=Lax; Path=/",
            session_id.0
        ))
        .map_err(|_| ApiError::Internal)?,
    );
    Ok((response_headers, Json(public_game(&game))))
}

async fn attempt(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<AttemptRequest>,
) -> Result<(HeaderMap, Json<AttemptResponse>), ApiError> {
    let mut repository = state.repository.lock().map_err(|_| ApiError::Internal)?;
    let (response, winning_session) = {
        let game = authorized_game_mut(&mut repository, &game_id, &headers)?;
        let was_won = game.status == GameStatus::Won;
        let attempt = state.engine.attempt(game, &request.word, unix_seconds().to_string())?;
        let now_won = game.status == GameStatus::Won;
        let response = AttemptResponse {
            attempt,
            status: game.status,
            attempt_count: game.attempts.len(),
            score: game.score,
        };
        let winning_session =
            (now_won && !was_won).then(|| (game.session_id.0.clone(), game.attempts.len() as u32));
        (response, winning_session)
    };
    if let Some((session_id, attempts)) = winning_session {
        let stats = repository.store.stats.entry(session_id).or_default();
        stats.games_played = stats.games_played.saturating_add(1);
        stats.games_won = stats.games_won.saturating_add(1);
        stats.current_streak = stats.current_streak.saturating_add(1);
        stats.best_streak = stats.best_streak.max(stats.current_streak);
        stats.total_attempts = stats.total_attempts.saturating_add(attempts);
    }
    repository.persist()?;
    Ok((cors_headers(), Json(response)))
}

async fn hint(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<HintResponse>), ApiError> {
    let mut repository = state.repository.lock().map_err(|_| ApiError::Internal)?;
    let game = authorized_game_mut(&mut repository, &game_id, &headers)?;
    let hint = state.engine.hint(game)?;
    let response = HintResponse { hint, hints_used: game.hints_used };
    repository.persist()?;
    Ok((cors_headers(), Json(response)))
}

async fn abandon(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<PublicGame>), ApiError> {
    let mut repository = state.repository.lock().map_err(|_| ApiError::Internal)?;
    let (session_id, response) = {
        let game = authorized_game_mut(&mut repository, &game_id, &headers)?;
        state.engine.abandon(game)?;
        (game.session_id.0.clone(), public_game(game))
    };
    let stats = repository.store.stats.entry(session_id).or_default();
    stats.games_played = stats.games_played.saturating_add(1);
    stats.current_streak = 0;
    repository.persist()?;
    Ok((cors_headers(), Json(response)))
}

async fn game(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<PublicGame>), ApiError> {
    let repository = state.repository.lock().map_err(|_| ApiError::Internal)?;
    let game = authorized_game(&repository, &game_id, &headers)?;
    Ok((cors_headers(), Json(public_game(game))))
}

async fn stats(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<StatsResponse>), ApiError> {
    let session_id = session_from_headers(&headers).ok_or(ApiError::Forbidden)?;
    let repository = state.repository.lock().map_err(|_| ApiError::Internal)?;
    let stats = repository.store.stats.get(&session_id.0).cloned().unwrap_or_default();
    let average_attempts = stats.average_attempts();
    Ok((cors_headers(), Json(StatsResponse { stats, average_attempts })))
}

async fn modes() -> (HeaderMap, Json<Vec<&'static str>>) {
    (cors_headers(), Json(vec!["daily", "infinite", "semantic_path"]))
}

fn authorized_game<'a>(
    repository: &'a Repository,
    game_id: &str,
    headers: &HeaderMap,
) -> Result<&'a GameSession, ApiError> {
    let session_id = session_from_headers(headers).ok_or(ApiError::Forbidden)?;
    let game = repository.store.games.get(game_id).ok_or(ApiError::NotFound)?;
    if game.session_id != session_id {
        return Err(ApiError::Forbidden);
    }
    Ok(game)
}

fn authorized_game_mut<'a>(
    repository: &'a mut Repository,
    game_id: &str,
    headers: &HeaderMap,
) -> Result<&'a mut GameSession, ApiError> {
    let session_id = session_from_headers(headers).ok_or(ApiError::Forbidden)?;
    let game = repository.store.games.get_mut(game_id).ok_or(ApiError::NotFound)?;
    if game.session_id != session_id {
        return Err(ApiError::Forbidden);
    }
    Ok(game)
}

fn public_game(game: &GameSession) -> PublicGame {
    PublicGame {
        game_id: game.id.0.clone(),
        challenge_id: game.challenge.id.0.clone(),
        date: game.challenge.date.clone(),
        language: game.challenge.language.clone(),
        mode: "daily",
        status: game.status,
        attempts: game.attempts.clone(),
        hints_used: game.hints_used,
        score: game.score,
        target_word: (game.status != GameStatus::InProgress)
            .then(|| game.challenge.target_word.clone()),
    }
}

fn session_from_headers(headers: &HeaderMap) -> Option<SessionId> {
    let cookie = headers.get(header::COOKIE)?.to_str().ok()?;
    cookie.split(';').map(str::trim).find_map(|entry| {
        entry.strip_prefix("lexicon_session=").map(|value| SessionId(value.to_owned()))
    })
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn today_utc() -> String {
    let days = (unix_seconds() / 86_400) as i64;
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}")
}

// Howard Hinnant's public-domain civil calendar conversion, with Unix epoch days as input.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    (year + if month <= 2 { 1 } else { 0 }, month as u32, day as u32)
}
