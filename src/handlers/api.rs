use crate::database::{
    AppState, get_all_veins, insert_vein_confirmation, insert_vein_depletion,
    insert_vein_revocation,
};
use crate::models::Vein;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::Redirect,
};

pub async fn get_veins_all(State(state): State<AppState>) -> Result<Json<Vec<Vein>>, StatusCode> {
    match get_all_veins(&state.db_pool).await {
        Ok(veins) => Ok(Json(veins)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn vein_comfirmation_set(
    State(state): State<AppState>,
    Path(vein_id): Path<String>,
) -> Result<Redirect, StatusCode> {
    match insert_vein_confirmation(&state.db_pool, &vein_id, true).await {
        Ok(_) => Ok(Redirect::to("/search")),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn vein_confirmation_revoke(
    State(state): State<AppState>,
    Path(vein_id): Path<String>,
) -> Result<Redirect, StatusCode> {
    match insert_vein_confirmation(&state.db_pool, &vein_id, false).await {
        Ok(_) => Ok(Redirect::to("/search")),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn vein_depletion_set(
    State(state): State<AppState>,
    Path(vein_id): Path<String>,
) -> Result<Redirect, StatusCode> {
    match insert_vein_depletion(&state.db_pool, &vein_id, true).await {
        Ok(_) => Ok(Redirect::to("/search")),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn vein_depletion_revoke(
    State(state): State<AppState>,
    Path(vein_id): Path<String>,
) -> Result<Redirect, StatusCode> {
    match insert_vein_depletion(&state.db_pool, &vein_id, false).await {
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
