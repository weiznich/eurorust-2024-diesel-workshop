//! Admin page setup for starts
use crate::app_state::{self, AppState};
use crate::database::schema::{categories, participants, races, starts};
use crate::database::Id;
use crate::errors::{Error, Result};
use axum::extract::Path;
use axum::response::{Html, Redirect};
use axum::{Form, Router};
use diesel::sqlite::Sqlite;
use diesel::{dsl, prelude::*};
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

#[derive(Serialize, Selectable, Queryable)]
#[diesel(table_name = starts)]
#[diesel(check_for_backend(Sqlite))]
struct StartData {
    id: Id,
    name: String,
    time: PrimitiveDateTime,
    #[diesel(select_expression = dsl::count_distinct(categories::id.nullable()))]
    category_count: i64,
    #[diesel(select_expression = dsl::count(participants::id.nullable()))]
    participant_count: i64,
}

#[axum::debug_handler(state = app_state::State)]
async fn list_starts_per_race(state: AppState, race_id: Path<Id>) -> Result<Html<String>> {
    let starts = state
        .with_connection(move |conn| {
            starts::table
                .left_join(categories::table.left_join(participants::table))
                .filter(starts::race_id.eq(race_id.0))
                .group_by(starts::id)
                .select(StartData::as_select())
                .load(conn)
        })
        .await?;
    state.render_template(
        "admin_list_starts.html",
        ListStartData {
            race_id: race_id.0,
            starts,
        },
    )
}

#[derive(Serialize, Queryable, Selectable)]
#[diesel(table_name = starts)]
#[diesel(check_for_backend(Sqlite))]
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
    state
        .with_connection(move |conn| {
            races::table
                .find(race_id.0)
                .select(races::id)
                .get_result::<Id>(conn)
                .optional()
        })
        .await?
        .ok_or_else(|| Error::NotFound(format!("Race with id {} not found", race_id.0)))?;
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

#[derive(Insertable, AsChangeset, Deserialize, Debug)]
#[diesel(table_name = starts)]
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
    state
        .with_connection(move |conn| {
            diesel::insert_into(starts::table)
                .values((data.0, starts::race_id.eq(race_id.0)))
                .execute(conn)
        })
        .await?;

    Ok(Redirect::to(&format!(
        "{base_url}/admin/races/{}/starts.html",
        race_id.0
    )))
}

#[axum::debug_handler(state = app_state::State)]
async fn delete_start(state: AppState, start_id: Path<Id>) -> Result<Redirect> {
    let base_url = state.base_url();
    let race_id = state
        .with_connection(move |conn| {
            diesel::delete(starts::table.find(start_id.0))
                .returning(starts::race_id)
                .get_result::<Id>(conn)
                .optional()
        })
        .await?
        .ok_or_else(|| Error::NotFound(format!("Start with id {} not found", start_id.0)))?;
    Ok(Redirect::to(&format!(
        "{base_url}/admin/races/{}/starts.html",
        race_id,
    )))
}

#[axum::debug_handler(state = app_state::State)]
async fn render_edit_start(state: AppState, start_id: Path<Id>) -> Result<Html<String>> {
    let start = state
        .with_connection(move |conn| {
            starts::table
                .find(start_id.0)
                .select(EditStartData::as_select())
                .first(conn)
                .optional()
        })
        .await?
        .ok_or_else(|| Error::NotFound(format!("Start with id {} not found", start_id.0)))?;
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
    let race_id = state
        .with_connection(move |conn| {
            diesel::update(starts::table.find(start_id.0))
                .set(data.0)
                .returning(starts::race_id)
                .get_result::<Id>(conn)
                .optional()
        })
        .await?
        .ok_or_else(|| Error::NotFound(format!("No start with id {} found", start_id.0)))?;
    Ok(Redirect::to(&format!(
        "{base_url}/admin/races/{}/starts.html",
        race_id
    )))
}
