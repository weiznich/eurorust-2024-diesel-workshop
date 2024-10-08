//! Admin page setup for races
use crate::app_state::{self, AppState};
use crate::database::schema::{
    categories, competitions, participants, races, special_categories, starts,
};
use crate::database::Id;
use crate::errors::{Error, Result};
use axum::extract::Path;
use axum::response::{Html, Redirect};
use axum::{Form, Router};
use diesel::dsl;
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use diesel::QueryDsl;
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

#[derive(Serialize, Selectable, Queryable)]
#[diesel(table_name = races)]
#[diesel(check_for_backend(Sqlite))]
struct RaceData {
    id: Id,
    name: String,
    #[diesel(select_expression = dsl::count_distinct(starts::id.nullable()))]
    starts: i64,
    #[diesel(select_expression = dsl::count(participants::id.nullable()))]
    participants: i64,
    #[diesel(
        select_expression =
            special_categories::table
            .filter(special_categories::race_id.eq(races::id))
            .select(dsl::count(special_categories::id))
            .single_value()
            .assume_not_null()
    )]
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
    let data = state
        .with_connection(move |conn| {
            races::table
                .filter(races::competition_id.eq(competition_id.0))
                .left_join(
                    starts::table.left_join(categories::table.left_join(participants::table)),
                )
                .group_by(races::id)
                .select(RaceData::as_select())
                .load(conn)
        })
        .await?;

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
    let _ = state
        .with_connection(move |conn| {
            competitions::table
                .find(competition_id.0)
                .select(competitions::id)
                .first::<Id>(conn)
                .optional()
        })
        .await?
        .ok_or_else(|| {
            Error::NotFound(format!(
                "Competition with id {} not found",
                competition_id.0
            ))
        })?;
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
    let competition_id = state
        .with_connection(move |conn| {
            diesel::delete(races::table.find(race_id.0))
                .returning(races::competition_id)
                .get_result::<Id>(conn)
                .optional()
        })
        .await?
        .ok_or_else(|| Error::NotFound(format!("Race with id {} not found", race_id.0)))?;
    Ok(Redirect::to(&format!(
        "{base_url}/admin/competitions/{}/races.html",
        competition_id
    )))
}

#[derive(Serialize, Queryable, Selectable)]
#[diesel(table_name = races)]
#[diesel(check_for_backend(Sqlite))]
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
    let race_data = state
        .with_connection(move |conn| {
            races::table
                .find(race_id.0)
                .select(EditRaceData::as_select())
                .first(conn)
                .optional()
        })
        .await?
        .ok_or_else(|| Error::NotFound(format!("Race with id {} not found", race_id.0)))?;
    state.render_template(
        "edit_race.html",
        RaceFormData {
            title: state.translation("edit_race"),
            target_url: format!("races/{}", race_id.0),
            race: Some(race_data),
        },
    )
}

#[derive(Deserialize, AsChangeset, Insertable)]
#[diesel(table_name = races)]
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
    let competition_id = state
        .with_connection(move |conn| {
            diesel::update(races::table.find(race_id.0))
                .set(data.0)
                .returning(races::competition_id)
                .get_result::<Id>(conn)
                .optional()
        })
        .await?
        .ok_or_else(|| Error::NotFound(format!("No race with id {} found", race_id.0)))?;
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
    state
        .with_connection(move |conn| {
            diesel::insert_into(races::table)
                .values((races::competition_id.eq(competition_id.0), data.0))
                .execute(conn)
        })
        .await?;
    Ok(Redirect::to(&format!(
        "{base_url}/admin/competitions/{}/races.html",
        competition_id.0
    )))
}
