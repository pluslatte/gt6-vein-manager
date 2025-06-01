use axum::{Json, extract::State, http::StatusCode};
use crate::database::{AppState, get_all_veins};
use crate::models::Vein;

pub async fn get_veins_all(State(state): State<AppState>) -> Result<Json<Vec<Vein>>, StatusCode> {
    match get_all_veins(&state.db_pool).await {
        Ok(veins) => Ok(Json(veins)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
