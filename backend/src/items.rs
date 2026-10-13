use crate::ai::Model;
use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, sqlx::FromRow)]
pub struct Item {
    pub id: i64,
    pub name: String,
    pub place: String,
    pub qty: f64,
    pub unit: String,
    pub cook_unit: Option<String>,
    pub cook_per_unit: Option<f64>,
    pub expires_on: Option<String>,
    pub expiry_guessed: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NewItem {
    pub name: String,
    pub place: String,
    pub qty: f64,
    pub unit: String,
    pub cook_unit: Option<String>,
    pub cook_per_unit: Option<f64>,
    pub expires_on: Option<String>,
    #[serde(default = "yes")]
    pub expiry_guessed: bool,
}

fn yes() -> bool {
    true
}

#[derive(Deserialize)]
pub struct AddRequest {
    pub items: Vec<NewItem>,
    pub source: String,
}

#[derive(Deserialize)]
pub struct ParseRequest {
    pub text: String,
}

#[derive(Serialize, Deserialize)]
pub struct Parsed {
    pub items: Vec<NewItem>,
}

type ApiError = (StatusCode, String);

fn internal(e: impl std::fmt::Display) -> ApiError {
    tracing::error!("{e}");
    (StatusCode::INTERNAL_SERVER_ERROR, "internal error".into())
}

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<Item>>, ApiError> {
    let items = sqlx::query_as::<_, Item>(
        "SELECT id, name, place, qty, unit, cook_unit, cook_per_unit, expires_on, expiry_guessed FROM items WHERE qty > 0 ORDER BY expires_on IS NULL, expires_on, name",
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;
    Ok(Json(items))
}

pub async fn add(State(state): State<AppState>, Json(req): Json<AddRequest>) -> Result<Json<Vec<i64>>, ApiError> {
    if !["receipt", "text", "photo", "manual"].contains(&req.source.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "unknown source".into()));
    }
    let mut tx = state.db.begin().await.map_err(internal)?;
    let mut ids = Vec::new();
    for it in &req.items {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO items (name, place, qty, unit, cook_unit, cook_per_unit, expires_on, expiry_guessed) VALUES (?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
        )
        .bind(&it.name)
        .bind(&it.place)
        .bind(it.qty)
        .bind(&it.unit)
        .bind(&it.cook_unit)
        .bind(it.cook_per_unit)
        .bind(&it.expires_on)
        .bind(it.expiry_guessed)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
        sqlx::query("INSERT INTO movements (item_id, delta, source) VALUES (?, ?, ?)")
            .bind(id)
            .bind(it.qty)
            .bind(&req.source)
            .execute(&mut *tx)
            .await
            .map_err(internal)?;
        if let (Some(cu), Some(f)) = (&it.cook_unit, it.cook_per_unit) {
            sqlx::query("INSERT INTO unit_factors (name, unit, cook_unit, cook_per_unit) VALUES (lower(?), ?, ?, ?) ON CONFLICT (name, unit) DO UPDATE SET cook_unit = excluded.cook_unit, cook_per_unit = excluded.cook_per_unit")
                .bind(&it.name)
                .bind(&it.unit)
                .bind(cu)
                .bind(f)
                .execute(&mut *tx)
                .await
                .map_err(internal)?;
        }
        ids.push(id);
    }
    tx.commit().await.map_err(internal)?;
    Ok(Json(ids))
}

const PARSE_SYSTEM: &str = "You turn a person's grocery note into pantry items. Reply with JSON only, no prose, shaped as {\"items\":[{\"name\":string,\"place\":\"fridge\"|\"freezer\"|\"cupboard\",\"qty\":number,\"unit\":string,\"cook_unit\":string|null,\"cook_per_unit\":number|null,\"expires_on\":\"YYYY-MM-DD\"|null,\"expiry_guessed\":true}]}. unit is the unit as bought (g, kg, ml, l, pack, piece). cook_unit is how a cook counts it (piece, clove, slice) and cook_per_unit how many cook units one unit holds, or null when the same. Guess the place and a typical expiry from today's date for that food stored there. Keep names short and singular, in English.";

pub async fn parse(State(state): State<AppState>, Json(req): Json<ParseRequest>) -> Result<Json<Parsed>, ApiError> {
    let text = req.text.trim();
    if text.is_empty() || text.len() > 4000 {
        return Err((StatusCode::BAD_REQUEST, "text must be 1 to 4000 characters".into()));
    }
    let known: Vec<(String, String, String, f64)> = sqlx::query_as("SELECT name, unit, cook_unit, cook_per_unit FROM unit_factors ORDER BY name LIMIT 200")
        .fetch_all(&state.db)
        .await
        .map_err(internal)?;
    let known = known.iter().map(|(n, u, c, f)| format!("{n}: 1 {u} = {f} {c}")).collect::<Vec<_>>().join("\n");
    let today: String = sqlx::query_scalar("SELECT date('now', 'localtime')").fetch_one(&state.db).await.map_err(internal)?;
    let prompt = format!("Today is {today}.\nUnit conversions the person already confirmed:\n{known}\n\nGrocery note:\n{text}");
    let parsed: Parsed = state.claude.ask(Model::Haiku, PARSE_SYSTEM, &prompt).await.map_err(|e| {
        tracing::error!("parse failed: {e:#}");
        (StatusCode::BAD_GATEWAY, "Claude could not read that, try again".into())
    })?;
    Ok(Json(parsed))
}
