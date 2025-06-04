use crate::database::{
    AppState, insert_vein_confirmation, insert_vein_depletion, insert_vein_revocation,
};
use axum::{
    Form,
    extract::{Path, State},
    http::StatusCode,
    response::Redirect,
};
use diesel_async::AsyncMysqlConnection;
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

#[derive(Debug)]
pub enum Action {
    Confirmation,
    Depletion,
    Revocation,
}

async fn handle_vein_action(
    _state: AppState,
    vein_id: String,
    form: VeinButtonForm,
    connection: &mut AsyncMysqlConnection,
    action: Action,
    status: bool,
) -> Result<Redirect, StatusCode> {
    let result = match action {
        Action::Confirmation => insert_vein_confirmation(connection, &vein_id, status).await,
        Action::Depletion => insert_vein_depletion(connection, &vein_id, status).await,
        Action::Revocation => insert_vein_revocation(connection, &vein_id, status).await,
    };

    match result {
        Ok(_) => {
            println!(
                "Vein action '{}' for vein ID '{}' was successful.",
                match action {
                    Action::Confirmation => "Confirmation",
                    Action::Depletion => "Depletion",
                    Action::Revocation => "Revocation",
                },
                vein_id
            );
            Ok(Redirect::to(&form.build_redirect_url()))
        }
        Err(_) => {
            eprintln!(
                "Failed to perform action '{}' for vein ID '{}'.",
                match action {
                    Action::Confirmation => "Confirmation",
                    Action::Depletion => "Depletion",
                    Action::Revocation => "Revocation",
                },
                vein_id
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

macro_rules! define_vein_action {
    ($func_name:ident, $action:expr, $status:expr) => {
        pub async fn $func_name(
            State(state): State<AppState>,
            Path(vein_id): Path<String>,
            Form(form): Form<VeinButtonForm>,
        ) -> Result<Redirect, StatusCode> {
            let mut connection = state
                .diesel_pool
                .get()
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            handle_vein_action(state, vein_id, form, &mut connection, $action, $status).await
        }
    };
}

// 定義されたマクロを使って関数を生成
define_vein_action!(vein_confirmation_set, Action::Confirmation, true);
define_vein_action!(vein_confirmation_revoke, Action::Confirmation, false);
define_vein_action!(vein_depletion_set, Action::Depletion, true);
define_vein_action!(vein_depletion_revoke, Action::Depletion, false);
define_vein_action!(vein_revocation_set, Action::Revocation, true);
define_vein_action!(vein_revocation_revoke, Action::Revocation, false);
