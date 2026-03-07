use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::{get, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fs, sync::Arc};
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

// PORT: DATA_DIR — set NOTES_DATA_DIR to relocate notes.json
fn notes_file() -> String {
    match std::env::var("NOTES_DATA_DIR") {
        Ok(dir) if !dir.is_empty() => format!("{dir}/notes.json"),
        _ => "notes.json".to_string(),
    }
}

// ── Data model ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Note {
    id: Uuid,
    title: String,
    body: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct CreateNote { title: String, body: String }

#[derive(Debug, Deserialize)]
struct UpdateNote { title: Option<String>, body: Option<String> }

#[derive(Clone)]
struct AppState {
    notes: Arc<Mutex<Vec<Note>>>,
}

// ── Persistence ─────────────────────────────────────────────────────────────

fn load_notes() -> Vec<Note> {
    match fs::read_to_string(notes_file()) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn save_notes(notes: &[Note]) {
    if let Ok(data) = serde_json::to_string_pretty(notes) {
        let _ = fs::write(notes_file(), data);
    }
}

// ── Handlers ────────────────────────────────────────────────────────────────

async fn get_notes_page() -> Html<&'static str> { Html(include_str!("notes.html")) }
async fn redirect_root() -> Redirect { Redirect::permanent("/notes") }

async fn list_notes(State(s): State<AppState>) -> Json<Vec<Note>> {
    Json(s.notes.lock().await.clone())
}

async fn create_note(State(s): State<AppState>, Json(p): Json<CreateNote>) -> impl IntoResponse {
    let now = Utc::now();
    let note = Note { id: Uuid::new_v4(), title: p.title, body: p.body, created_at: now, updated_at: now };
    let mut notes = s.notes.lock().await;
    notes.push(note.clone());
    let snap = notes.clone(); drop(notes);
    save_notes(&snap);
    (StatusCode::CREATED, Json(note))
}

async fn update_note(State(s): State<AppState>, Path(id): Path<Uuid>, Json(p): Json<UpdateNote>) -> impl IntoResponse {
    let mut notes = s.notes.lock().await;
    let Some(note) = notes.iter_mut().find(|n| n.id == id) else {
        return (StatusCode::NOT_FOUND, Json(None::<Note>)).into_response();
    };
    if let Some(t) = p.title { note.title = t; }
    if let Some(b) = p.body  { note.body = b; }
    note.updated_at = Utc::now();
    let updated = note.clone();
    let snap = notes.clone(); drop(notes);
    save_notes(&snap);
    (StatusCode::OK, Json(Some(updated))).into_response()
}

async fn delete_note(State(s): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let mut notes = s.notes.lock().await;
    let before = notes.len();
    notes.retain(|n| n.id != id);
    if notes.len() == before { return StatusCode::NOT_FOUND; }
    let snap = notes.clone(); drop(notes);
    save_notes(&snap);
    StatusCode::NO_CONTENT
}

// ── Main ────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // PORT: PORT — HTTP listen port, set by EPC via the PORT env var
    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let bind_addr = format!("{host}:{port}");

    // PORT: DATA_DIR — set NOTES_DATA_DIR to move notes.json elsewhere
    let state = AppState {
        notes: Arc::new(Mutex::new(load_notes())),
    };

    let app = Router::new()
        .route("/", get(redirect_root))
        .route("/notes", get(get_notes_page))
        .route("/api/notes", get(list_notes).post(create_note))
        .route("/api/notes/{id}", put(update_note).delete(delete_note))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .expect("failed to bind address");

    println!("notes running at http://{}", bind_addr);
    axum::serve(listener, app).await.unwrap();
}
