use crate::database::{
    AppState, insert_vein_confirmation, insert_vein_depletion, insert_vein_revocation,
};
use axum::{
    Form,
    extract::{Path, State},
    http::StatusCode,
    response::Redirect,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct VeinButtonForm {
    query_state: Option<String>,
}

impl VeinButtonForm {
    fn build_redirect_url(&self) -> String {
        let mut url = String::from("/search");
        if let Some(query_state) = &self.query_state {
            if !query_state.is_empty() {
                url.push_str(&format!("?{}", query_state));
            }
        }
        url
    }
}

pub async fn vein_comfirmation_set(
    State(state): State<AppState>,
    Path(vein_id): Path<String>,
    Form(form): Form<VeinButtonForm>,
) -> Result<Redirect, StatusCode> {
    match insert_vein_confirmation(&state.db_pool, &vein_id, true).await {
        Ok(_) => Ok(Redirect::to(&form.build_redirect_url())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn vein_confirmation_revoke(
    State(state): State<AppState>,
    Path(vein_id): Path<String>,
    Form(form): Form<VeinButtonForm>,
) -> Result<Redirect, StatusCode> {
    match insert_vein_confirmation(&state.db_pool, &vein_id, false).await {
        Ok(_) => Ok(Redirect::to(&form.build_redirect_url())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn vein_depletion_set(
    State(state): State<AppState>,
    Path(vein_id): Path<String>,
    Form(form): Form<VeinButtonForm>,
) -> Result<Redirect, StatusCode> {
    match insert_vein_depletion(&state.db_pool, &vein_id, true).await {
        Ok(_) => Ok(Redirect::to(&form.build_redirect_url())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn vein_depletion_revoke(
    State(state): State<AppState>,
    Path(vein_id): Path<String>,
    Form(form): Form<VeinButtonForm>,
) -> Result<Redirect, StatusCode> {
    match insert_vein_depletion(&state.db_pool, &vein_id, false).await {
        Ok(_) => Ok(Redirect::to(&form.build_redirect_url())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn vein_revocation_set(
    State(state): State<AppState>,
    Path(vein_id): Path<String>,
    Form(form): Form<VeinButtonForm>,
) -> Result<Redirect, StatusCode> {
    match insert_vein_revocation(&state.db_pool, &vein_id, true).await {
        Ok(_) => Ok(Redirect::to(&form.build_redirect_url())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn vein_revocation_revoke(
    State(state): State<AppState>,
    Path(vein_id): Path<String>,
    Form(form): Form<VeinButtonForm>,
) -> Result<Redirect, StatusCode> {
    match insert_vein_revocation(&state.db_pool, &vein_id, false).await {
        Ok(_) => Ok(Redirect::to(&form.build_redirect_url())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
