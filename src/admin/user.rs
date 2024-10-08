//! Admin page setup for users
use crate::app_state::{self, AppState};
use crate::errors::Result;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect};
use axum::Form;
use serde::Deserialize;

pub mod auth_session;
pub mod sqlite_session_store;

#[derive(Clone, Deserialize)]
pub struct Credentials {
    name: String,
    password: String,
}

/// Handler for rendering the login page
#[axum::debug_handler(state = app_state::State)]
pub async fn login_form(state: AppState) -> Result<Html<String>> {
    state.render_template("login.html", ())
}

/// Handler for handling the form data from the login page
/// This is where the actual login happens
#[axum::debug_handler]
pub async fn handle_login(
    state: State<app_state::State>,
    mut auth_session: self::auth_session::AuthSession,
    Form(creds): Form<Credentials>,
) -> impl IntoResponse {
    let user = match auth_session.authenticate(creds.clone()).await {
        Ok(Some(user)) => user,
        Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if auth_session.login(&user).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    let base_url = &state.base_url;
    Redirect::to(&format!("{base_url}/admin/competitions/index.html")).into_response()
}
