use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, patch, post},
    Json, Router,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
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
        .route("/players/:id", get(get_player))
        .route("/players/:id", patch(update_player))
        .with_state(Arc::clone(&shared_state));

    // listen globally on port 8000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn create_player(
    State(state): State<SharedState>,
    Json(payload): Json<CreatePlayer>,
) -> (StatusCode, Result<Json<Player>, Json<Error>>) {
    let state = &mut state.write().unwrap();

    if state.names.contains(&payload.name) {
        return (
            StatusCode::FORBIDDEN,
            Err(Json(Error {
                error: "Name is already in use".to_string(),
            })),
        );
    }

    // insert your application logic here
    let player = Player {
        id: rand::thread_rng().gen(),
        name: payload.name,
        position: Default::default(),
    };

    state.players.insert(player.id, player.clone());
    state.names.insert(player.name.clone());

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Ok(Json(player)))
}

async fn get_players(State(state): State<SharedState>) -> Result<Json<Vec<Player>>, StatusCode> {
    let players = &state.read().unwrap().players;

    Ok(Json(
        players.values().map(|player| player.clone()).collect(),
    ))
}

async fn get_player(
    Path(id): Path<u64>,
    State(state): State<SharedState>,
) -> Result<Json<Player>, StatusCode> {
    let players = &state.read().unwrap().players;

    if let Some(player) = players.get(&id) {
        Ok(Json(player.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn update_player(
    Path(id): Path<u64>,
    State(state): State<SharedState>,
    Json(payload): Json<Position>,
) -> Result<Json<Player>, StatusCode> {
    let players = &mut state.write().unwrap().players;

    if let Some(player) = players.get_mut(&id) {
        player.position = payload;
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
    id: u64,
    name: String,
    position: Position,
}

#[derive(Serialize, Clone)]
struct Error {
    error: String,
}

type SharedState = Arc<RwLock<GameState>>;

#[derive(Default)]
struct GameState {
    players: HashMap<u64, Player>,
    names: HashSet<String>,
}
