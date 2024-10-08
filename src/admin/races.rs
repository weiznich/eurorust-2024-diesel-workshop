//! Admin page setup for races
use crate::app_state::{self, AppState};
use crate::database::Id;
use crate::errors::Result;
use axum::extract::Path;
use axum::response::{Html, Redirect};
use axum::{Form, Router};
use serde::{Deserialize, Serialize};

pub fn routes() -> Router<app_state::State> {
    let races_router = Router::new()
        .route("/:race_id/delete.html", axum::routing::get(delete_race))
        .route("/:race_id/edit.html", axum::routing::get(render_edit_race))
        .route("/:race_id", axum::routing::post(update_race));
    Router::new()
        .nest("/races/", races_router)
        .route(
            "/competitions/:competition_id/races.html",
            axum::routing::get(list_races_for_competition),
        )
        .route(
            "/competitions/:competition_id/new_race.html",
            axum::routing::get(render_new_race),
        )
        .route(
            "/competitions/:competition_id/new_race",
            axum::routing::post(new_race),
        )
}

#[derive(Serialize)]
struct RaceData {
    id: Id,
    name: String,
    starts: i64,
    participants: i64,
    special_categories: i64,
}

#[derive(Serialize)]
struct ListRaceData {
    races: Vec<RaceData>,
    competition_id: Id,
}

#[axum::debug_handler(state = app_state::State)]
async fn list_races_for_competition(
    state: AppState,
    competition_id: Path<Id>,
) -> Result<Html<String>> {
    let data: Vec<RaceData> = todo!("Load all relevant race data for a competition");

    state.render_template(
        "admin_list_races.html",
        ListRaceData {
            races: data,
            competition_id: competition_id.0,
        },
    )
}

#[axum::debug_handler(state = app_state::State)]
async fn render_new_race(state: AppState, competition_id: Path<Id>) -> Result<Html<String>> {
    todo!("Check if the competition exists");
    state.render_template(
        "edit_race.html",
        RaceFormData {
            race: None,
            title: state.translation("new_race"),
            target_url: format!("competitions/{}/new_race", competition_id.0),
        },
    )
}

#[axum::debug_handler(state = app_state::State)]
async fn delete_race(state: AppState, race_id: Path<Id>) -> Result<Redirect> {
    let base_url = state.base_url();
    let competition_id: Id = todo!("Delete the race + get the competition id");
    Ok(Redirect::to(&format!(
        "{base_url}/admin/competitions/{}/races.html",
        competition_id
    )))
}

#[derive(Serialize)]
struct EditRaceData {
    id: Id,
    name: String,
    competition_id: Id,
}

#[derive(Serialize)]
struct RaceFormData {
    race: Option<EditRaceData>,
    title: String,
    target_url: String,
}

#[axum::debug_handler(state = app_state::State)]
async fn render_edit_race(state: AppState, race_id: Path<Id>) -> Result<Html<String>> {
    let race_data: EditRaceData = todo!("Get data for a specific race");
    state.render_template(
        "edit_race.html",
        RaceFormData {
            title: state.translation("edit_race"),
            target_url: format!("races/{}", race_id.0),
            race: Some(race_data),
        },
    )
}

#[derive(Deserialize)]
struct RaceFormInput {
    name: String,
}

#[axum::debug_handler(state = app_state::State)]
async fn update_race(
    state: AppState,
    race_id: Path<Id>,
    data: Form<RaceFormInput>,
) -> Result<Redirect> {
    let base_url = state.base_url();
    let competition_id: Id = todo!("Update the race + get the competition id");
    Ok(Redirect::to(&format!(
        "{base_url}/admin/competitions/{}/races.html",
        competition_id
    )))
}

#[axum::debug_handler(state = app_state::State)]
async fn new_race(
    state: AppState,
    competition_id: Path<Id>,
    data: Form<RaceFormInput>,
) -> Result<Redirect> {
    let base_url = state.base_url();
    todo!("Create a new race");
    Ok(Redirect::to(&format!(
        "{base_url}/admin/competitions/{}/races.html",
        competition_id.0
    )))
}
