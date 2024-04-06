use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, patch, post},
    Json, Router,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // initialize game state
    let shared_state = SharedState::default();

    // build our application with a route
    let app = Router::new()
        // a static file server
        .nest_service("/", ServeDir::new("dist"))
        // players
        .route("/players", post(create_player))
        .route("/players", get(get_players))
        .route("/players/:name", get(get_player))
        .route("/players/:name", patch(update_player))
        .with_state(Arc::clone(&shared_state));

    // listen globally on port 8000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn create_player(
    State(state): State<SharedState>,
    Json(payload): Json<CreatePlayer>,
) -> (StatusCode, Result<Json<Registration>, Json<Error>>) {
    let state = &mut state.write().unwrap();

    if state.players.contains_key(&payload.name) {
        return (
            StatusCode::FORBIDDEN,
            Err(Json(Error {
                error: "Name is already in use".to_string(),
            })),
        );
    }

    // Create player
    let player = Player {
        name: payload.name,
        position: Default::default(),
    };
    state.players.insert(player.name.clone(), player.clone());

    // Create secret and register
    let registration = Registration {
        player: player.clone(),
        secret: rand::thread_rng().gen(),
    };
    state.secrets.insert(player.name, registration.secret);

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Ok(Json(registration)))
}

async fn get_players(State(state): State<SharedState>) -> Result<Json<Vec<Player>>, StatusCode> {
    let players = &state.read().unwrap().players;

    Ok(Json(
        players.values().map(|player| player.clone()).collect(),
    ))
}

async fn get_player(
    Path(name): Path<String>,
    State(state): State<SharedState>,
) -> Result<Json<Player>, StatusCode> {
    let players = &state.read().unwrap().players;

    if let Some(player) = players.get(&name) {
        Ok(Json(player.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn update_player(
    Path(name): Path<String>,
    State(state): State<SharedState>,
    Json(payload): Json<PlayerUpdate>,
) -> Result<Json<Player>, StatusCode> {
    let state = &mut state.write().unwrap();

    if let Some(secret) = state.secrets.get(&name) {
        if *secret != payload.secret {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    if let Some(player) = state.players.get_mut(&name) {
        player.position = payload.position;
        Ok(Json(player.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[derive(Deserialize)]
struct CreatePlayer {
    name: String,
}

#[derive(Deserialize, Serialize, Default, Clone)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Serialize, Clone)]
struct Player {
    name: String,
    position: Position,
}

#[derive(Serialize, Clone)]
struct Registration {
    player: Player,
    secret: u64,
}

#[derive(Deserialize, Clone)]
struct PlayerUpdate {
    position: Position,
    secret: u64,
}

#[derive(Serialize, Clone)]
struct Error {
    error: String,
}

type SharedState = Arc<RwLock<GameState>>;

#[derive(Default)]
struct GameState {
    players: HashMap<String, Player>,
    secrets: HashMap<String, u64>,
}
