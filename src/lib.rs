// caused by the todo macros that
// are supposed to be replaced by workshop participants
#![allow(unreachable_code, unused_variables, dead_code)]
use admin::user::auth_session::LoginBackend;
use admin::user::sqlite_session_store::SqliteSessionStore;
use axum::http::header::CONTENT_TYPE;
use axum::http::HeaderValue;
use axum::response::{IntoResponse, Response};
use axum::Router;
use axum_login::tower_sessions::SessionManagerLayer;
use axum_login::AuthManagerLayerBuilder;
use diesel_migrations::MigrationHarness;
use service_config::Config;
use tower_http::trace::TraceLayer;

pub mod admin;
pub mod app_state;
mod competition_overview;
pub mod database;
pub mod errors;
mod registration;
mod registration_list;
pub mod service_config;

mod axum_ext;

const MIGRATIONS: diesel_migrations::EmbeddedMigrations = diesel_migrations::embed_migrations!();

pub async fn setup(config: Config) -> (Router, app_state::State) {
    let base_url = config.base_url.clone();
    let state = app_state::State::from_config(&config);

    let conn = state
        .pool
        .get()
        .await
        .expect("Failed to get a connection from the pool");
    conn.interact(|conn| conn.run_pending_migrations(MIGRATIONS).map(|_| ()))
        .await
        .expect("Failed to run migrations")
        .expect("Failed to run migrations");

    if config.insert_test_data {
        conn.interact(database::test_data::insert_test_data)
            .await
            .expect("Failed to insert test data")
            .expect("Failed to insert test data");
    }
    // Session layer.
    let session_store = SqliteSessionStore::new(state.pool.clone());
    let session_layer = SessionManagerLayer::new(session_store);

    // Auth service.
    let backend = LoginBackend::new(state.pool.clone());
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let router = Router::new()
        .route("/assets/simple.min.css", axum::routing::get(get_simple_css))
        .route("/assets/custom.css", axum::routing::get(get_custom_css))
        .route(
            "/index.html",
            axum::routing::get(self::competition_overview::render),
        )
        .merge(registration::routes())
        .merge(registration_list::routes())
        .nest("/admin", admin::routes());
    let router = if base_url.is_empty() {
        router
    } else {
        Router::new().nest(&base_url, router)
    };
    let router = router
        .layer(auth_layer)
        .with_state(state.clone())
        .layer(TraceLayer::new_for_http());
    (router, state)
}

async fn get_simple_css() -> Response {
    let mut resp = include_str!("../assets/simple.min.css").into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("text/css"));
    resp
}

async fn get_custom_css() -> Response {
    let mut resp = include_str!("../assets/custom.css").into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("text/css"));
    resp
}
