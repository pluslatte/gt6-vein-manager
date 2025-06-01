use axum::{Json, extract::{State, Path}, http::StatusCode, response::Redirect};
use crate::database::{AppState, get_all_veins, insert_vein_confirmation, insert_vein_depletion, insert_vein_revocation};
use crate::models::Vein;

pub async fn get_veins_all(State(state): State<AppState>) -> Result<Json<Vec<Vein>>, StatusCode> {
    match get_all_veins(&state.db_pool).await {
        Ok(veins) => Ok(Json(veins)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn update_vein_confirmation(
    State(state): State<AppState>,
    Path(vein_id): Path<String>,
) -> Result<Redirect, StatusCode> {
    match insert_vein_confirmation(&state.db_pool, &vein_id).await {
        Ok(_) => Ok(Redirect::to("/search")),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn update_vein_depletion(
    State(state): State<AppState>,
    Path(vein_id): Path<String>,
) -> Result<Redirect, StatusCode> {
    match insert_vein_depletion(&state.db_pool, &vein_id).await {
        Ok(_) => Ok(Redirect::to("/search")),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn update_vein_revocation(
    State(state): State<AppState>,
    Path(vein_id): Path<String>,
) -> Result<Redirect, StatusCode> {
    match insert_vein_revocation(&state.db_pool, &vein_id).await {
        Ok(_) => Ok(Redirect::to("/search")),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
