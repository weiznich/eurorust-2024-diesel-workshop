//! Admin page setup for starts
use crate::app_state::{self, AppState};
use crate::database::Id;
use crate::errors::Result;
use axum::extract::Path;
use axum::response::{Html, Redirect};
use axum::{Form, Router};
use serde::{Deserialize, Deserializer, Serialize};
use time::macros::format_description;
use time::PrimitiveDateTime;

pub fn routes() -> Router<app_state::State> {
    let start_routes = Router::new()
        .route("/:start_id/delete.html", axum::routing::get(delete_start))
        .route(
            "/:start_id/edit.html",
            axum::routing::get(render_edit_start),
        )
        .route("/:start_id", axum::routing::post(update_start));
    Router::new()
        .nest("/starts", start_routes)
        .route(
            "/races/:race_id/starts.html",
            axum::routing::get(list_starts_per_race),
        )
        .route(
            "/races/:race_id/create_start.html",
            axum::routing::get(render_create_start),
        )
        .route(
            "/races/:race_id/create_start",
            axum::routing::post(create_start),
        )
}

#[derive(Serialize)]
struct ListStartData {
    race_id: Id,
    starts: Vec<StartData>,
}

#[derive(Serialize)]
struct StartData {
    id: Id,
    name: String,
    time: PrimitiveDateTime,
    category_count: i64,
    participant_count: i64,
}

#[axum::debug_handler(state = app_state::State)]
async fn list_starts_per_race(state: AppState, race_id: Path<Id>) -> Result<Html<String>> {
    let starts: Vec<StartData> = todo!("Get all starts for a certain race");
    state.render_template(
        "admin_list_starts.html",
        ListStartData {
            race_id: race_id.0,
            starts,
        },
    )
}

#[derive(Serialize)]
struct EditStartData {
    name: String,
    time: PrimitiveDateTime,
    race_id: Id,
}

#[derive(Serialize)]
struct StartFormData {
    race_id: Id,
    start: Option<EditStartData>,
    target_url: String,
    title: String,
}

#[axum::debug_handler(state = app_state::State)]
async fn render_create_start(state: AppState, race_id: Path<Id>) -> Result<Html<String>> {
    todo!("Verify that the race exists");
    state.render_template(
        "edit_start.html",
        StartFormData {
            race_id: race_id.0,
            start: None,
            target_url: format!("races/{}/create_start", race_id.0),
            title: state.translation("new_start"),
        },
    )
}

#[derive(Deserialize, Debug)]
struct StartInputData {
    name: String,
    #[serde(deserialize_with = "parse_date")]
    time: PrimitiveDateTime,
}

fn parse_date<'de, D>(d: D) -> Result<PrimitiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s = <String as Deserialize>::deserialize(d)?;
    let format = format_description!("[year]-[month]-[day]T[hour]:[minute]");
    let out = PrimitiveDateTime::parse(&s, format)
        .map_err(|e| serde::de::Error::custom(e.to_string()))?;
    Ok(out)
}

#[axum::debug_handler(state = app_state::State)]
async fn create_start(
    state: AppState,
    race_id: Path<Id>,
    data: Form<StartInputData>,
) -> Result<Redirect> {
    let base_url = state.base_url();
    todo!("Insert a new start");

    Ok(Redirect::to(&format!(
        "{base_url}/admin/races/{}/starts.html",
        race_id.0
    )))
}

#[axum::debug_handler(state = app_state::State)]
async fn delete_start(state: AppState, start_id: Path<Id>) -> Result<Redirect> {
    let base_url = state.base_url();
    let race_id: Id = todo!("Delete the given start and get the corresponding race id");
    Ok(Redirect::to(&format!(
        "{base_url}/admin/races/{}/starts.html",
        race_id,
    )))
}

#[axum::debug_handler(state = app_state::State)]
async fn render_edit_start(state: AppState, start_id: Path<Id>) -> Result<Html<String>> {
    let start: EditStartData = todo!("Load all data to edit a given start");
    state.render_template(
        "edit_start.html",
        StartFormData {
            race_id: start.race_id,
            start: Some(start),
            target_url: format!("starts/{}", start_id.0),
            title: state.translation("edit_start"),
        },
    )
}

#[axum::debug_handler(state = app_state::State)]
async fn update_start(
    state: AppState,
    start_id: Path<Id>,
    data: Form<StartInputData>,
) -> Result<Redirect> {
    let base_url = state.base_url();
    let race_id: Id = todo!("Update the given start and return the id of the relevant race");
    Ok(Redirect::to(&format!(
        "{base_url}/admin/races/{}/starts.html",
        race_id
    )))
}
