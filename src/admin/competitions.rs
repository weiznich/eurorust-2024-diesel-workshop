//! Admin page setup for competitions

use crate::app_state::{self, AppState};
use crate::database::schema::{categories, competitions, participants, races, starts};
use crate::database::shared_models::Competition;
use crate::database::Id;
use crate::errors::Error;
use crate::errors::Result;
use axum::extract::Path;
use axum::response::Html;
use axum::response::Redirect;
use axum::{Form, Router};
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use diesel::{dsl, AsChangeset, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use time::Date;

pub fn routes() -> Router<app_state::State> {
    Router::new()
        .route("/index.html", axum::routing::get(list_competitions))
        .route(
            "/create.html",
            axum::routing::get(render_create_competition),
        )
        .route("/create", axum::routing::post(create_competition))
        .route("/:id/delete.html", axum::routing::get(delete_competition))
        .route(
            "/:id/edit.html",
            axum::routing::get(render_edit_competition),
        )
        .route("/:id", axum::routing::post(update_competition))
}

#[derive(Queryable, Selectable, Serialize, Debug)]
#[diesel(table_name = competitions::table)]
#[diesel(check_for_backend(Sqlite))]
pub(crate) struct CompetitionWithData {
    #[diesel(embed)]
    #[serde(flatten)]
    pub(crate) competition: Competition,
    #[diesel(select_expression = dsl::count_distinct(races::id.nullable()))]
    pub(crate) race_count: i64,
    #[diesel(select_expression = dsl::count(participants::id.nullable()))]
    pub(crate) participant_count: i64,
}

#[derive(Serialize, Debug)]
pub(crate) struct ListCompetitionData {
    pub(crate) competitions: Vec<CompetitionWithData>,
}

#[derive(Insertable, Deserialize, AsChangeset)]
#[diesel(table_name = competitions)]
pub(crate) struct NewCompetition {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) date: Date,
    pub(crate) location: String,
    pub(crate) announcement: String,
}

#[axum::debug_handler(state = app_state::State)]
pub(crate) async fn list_competitions(state: AppState) -> Result<Html<String>> {
    let competitions = state
        .with_connection(|conn| {
            competitions::table
                .left_join(races::table.left_join(
                    starts::table.left_join(categories::table.left_join(participants::table)),
                ))
                .group_by(competitions::id)
                .select(CompetitionWithData::as_select())
                .load(conn)
        })
        .await?;
    state.render_template(
        "admin_competition_list.html",
        ListCompetitionData { competitions },
    )
}

#[derive(Serialize)]
pub(crate) struct EditCompetitionData {
    pub(crate) competition: Option<Competition>,
    pub(crate) target_url: String,
    pub(crate) title: String,
}

#[axum::debug_handler(state = app_state::State)]
pub(crate) async fn render_create_competition(state: AppState) -> Result<Html<String>> {
    state.render_template(
        "create_competition.html",
        EditCompetitionData {
            competition: None,
            target_url: "competitions/create".into(),
            title: state.translation("new_competition"),
        },
    )
}

#[axum::debug_handler(state = app_state::State)]
pub(crate) async fn create_competition(
    state: AppState,
    data: Form<NewCompetition>,
) -> Result<Redirect> {
    let base_url = state.base_url();
    state
        .with_connection(|conn| {
            diesel::insert_into(competitions::table)
                .values(data.0)
                .execute(conn)
        })
        .await?;

    Ok(Redirect::to(&format!(
        "{base_url}/admin/competitions/index.html"
    )))
}

#[axum::debug_handler(state = app_state::State)]
pub(crate) async fn delete_competition(state: AppState, id: Path<Id>) -> Result<Redirect> {
    let base_url = state.base_url();
    let count = state
        .with_connection(move |conn| diesel::delete(competitions::table.find(id.0)).execute(conn))
        .await?;
    if count != 1 {
        Err(Error::NotFound(format!(
            "Competition with {} not found",
            id.0
        )))
    } else {
        Ok(Redirect::to(&format!(
            "{base_url}/admin/competitions/index.html"
        )))
    }
}

#[axum::debug_handler(state = app_state::State)]
pub(crate) async fn render_edit_competition(state: AppState, id: Path<Id>) -> Result<Html<String>> {
    let competition = state
        .with_connection(move |conn| {
            competitions::table
                .find(id.0)
                .select(Competition::as_select())
                .first(conn)
                .optional()
        })
        .await?
        .ok_or_else(|| Error::NotFound(format!("Competition with id {} not found", id.0)))?;
    state.render_template(
        "create_competition.html",
        EditCompetitionData {
            competition: Some(competition),
            target_url: format!("competitions/{}", id.0),
            title: state.translation("edit_competition"),
        },
    )
}

#[axum::debug_handler(state = app_state::State)]
pub(crate) async fn update_competition(
    state: AppState,
    id: Path<Id>,
    data: Form<NewCompetition>,
) -> Result<Redirect> {
    let base_url = state.base_url();
    let count = state
        .with_connection(move |conn| {
            diesel::update(competitions::table.find(id.0))
                .set(data.0)
                .execute(conn)
        })
        .await?;
    if count != 1 {
        Err(Error::NotFound(format!(
            "Competition with {} not found",
            id.0
        )))
    } else {
        Ok(Redirect::to(&format!(
            "{base_url}/admin/competitions/index.html"
        )))
    }
}
