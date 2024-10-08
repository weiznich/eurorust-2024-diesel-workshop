//! This module contains all admin page subroutes
//! Additionally it contains the code for the user authentification for
//! access to the admin pages
use crate::app_state::{self};
use axum::Router;
use axum_login::login_required;
use user::auth_session::LoginBackend;

mod categories;
mod competitions;
mod participants;
mod races;
mod special_categories;
mod starts;
/// User authentication for the admin pages
pub mod user;

pub fn routes() -> Router<app_state::State> {
    Router::new()
        .nest("/competitions", competitions::routes())
        .merge(participants::routes())
        .merge(races::routes())
        .merge(starts::routes())
        .merge(categories::routes())
        .merge(special_categories::routes())
        .route_layer(login_required!(
            LoginBackend,
            login_url = "/admin/login.html"
        ))
        .route("/login.html", axum::routing::get(user::login_form))
        .route("/login", axum::routing::post(user::handle_login))
}
