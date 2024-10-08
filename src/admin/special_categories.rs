//! Admin page setup for special_categories
use crate::app_state::{self, AppState};
use crate::database::schema::{
    participants, participants_in_special_category, races, special_categories,
};
use crate::database::Id;
use crate::errors::{Error, Result};
use axum::extract::Path;
use axum::response::{Html, Redirect};
use axum::{Form, Router};
use diesel::dsl;
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use serde::{Deserialize, Serialize};

pub(crate) fn routes() -> Router<app_state::State> {
    let special_categories_router = Router::new()
        .route(
            "/:special_id/delete.html",
            axum::routing::get(delete_special_category),
        )
        .route(
            "/:special_id/edit.html",
            axum::routing::get(render_edit_special_category),
        )
        .route("/:special_id", axum::routing::post(update_special_category));

    Router::new()
        .nest("/special_categories", special_categories_router)
        .route(
            "/races/:race_id/special_categories.html",
            axum::routing::get(list_special_categories),
        )
        .route(
            "/races/:race_id/new_special_category.html",
            axum::routing::get(render_add_special_category),
        )
        .route(
            "/races/:race_id/new_special_category",
            axum::routing::post(add_special_category),
        )
}

#[derive(Serialize, Queryable, Selectable)]
#[diesel(table_name = special_categories)]
#[diesel(check_for_backend(Sqlite))]
struct SpecialCategoryData {
    id: Id,
    short_name: String,
    name: String,
    #[diesel(select_expression = dsl::count(participants::id.nullable()))]
    participant_count: i64,
}

#[derive(Serialize)]
struct ListSpecialCategoriesData {
    special_categories: Vec<SpecialCategoryData>,
    race_id: Id,
}

#[axum::debug_handler(state = app_state::State)]
async fn list_special_categories(state: AppState, race_id: Path<Id>) -> Result<Html<String>> {
    let special_categories = state
        .with_connection(move |conn| {
            special_categories::table
                .left_join(participants_in_special_category::table.left_join(participants::table))
                .group_by(special_categories::id)
                .select(SpecialCategoryData::as_select())
                .filter(special_categories::race_id.eq(race_id.0))
                .load(conn)
        })
        .await?;
    state.render_template(
        "admin_list_special_categories.html",
        ListSpecialCategoriesData {
            special_categories,
            race_id: race_id.0,
        },
    )
}

#[axum::debug_handler(state = app_state::State)]
async fn delete_special_category(state: AppState, special_id: Path<Id>) -> Result<Redirect> {
    let base_url = state.base_url();
    let race_id = state
        .with_connection(move |conn| {
            diesel::delete(special_categories::table.find(special_id.0))
                .returning(special_categories::race_id)
                .get_result::<Id>(conn)
                .optional()
        })
        .await?
        .ok_or_else(|| {
            Error::NotFound(format!(
                "Special category with id {} not found",
                special_id.0
            ))
        })?;
    Ok(Redirect::to(&format!(
        "{base_url}/admin/races/{}/special_categories.html",
        race_id
    )))
}

#[derive(Serialize, Queryable, Selectable)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(table_name = special_categories)]
struct EditSpecialCategoriesData {
    short_name: String,
    name: String,
    race_id: Id,
}

#[derive(Serialize)]
struct SpecialCategoryFormData {
    race_id: Id,
    special_category: Option<EditSpecialCategoriesData>,
    target_url: String,
    title: String,
}

#[axum::debug_handler(state = app_state::State)]
async fn render_add_special_category(state: AppState, race_id: Path<Id>) -> Result<Html<String>> {
    state
        .with_connection(move |conn| {
            races::table
                .find(race_id.0)
                .select(races::id)
                .get_result::<Id>(conn)
                .optional()
        })
        .await?
        .ok_or_else(|| Error::NotFound(format!("No race with id {} found", race_id.0)))?;
    state.render_template(
        "edit_special_category.html",
        SpecialCategoryFormData {
            race_id: race_id.0,
            special_category: None,
            target_url: format!("races/{}/new_special_category", race_id.0),
            title: state.translation("new_special_category"),
        },
    )
}

#[axum::debug_handler(state = app_state::State)]
async fn render_edit_special_category(
    state: AppState,
    special_id: Path<Id>,
) -> Result<Html<String>> {
    let special_category = state
        .with_connection(move |conn| {
            special_categories::table
                .find(special_id.0)
                .select(EditSpecialCategoriesData::as_select())
                .first(conn)
                .optional()
        })
        .await?
        .ok_or_else(|| {
            Error::NotFound(format!(
                "No special category with id {} found",
                special_id.0
            ))
        })?;
    state.render_template(
        "edit_special_category.html",
        SpecialCategoryFormData {
            race_id: special_category.race_id,
            special_category: Some(special_category),
            target_url: format!("special_categories/{}", special_id.0),
            title: state.translation("edit_special_category"),
        },
    )
}

#[derive(Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = special_categories)]
struct SpecialCategoryFormInputData {
    short_name: String,
    name: String,
}

#[axum::debug_handler(state = app_state::State)]
async fn update_special_category(
    state: AppState,
    special_id: Path<Id>,
    data: Form<SpecialCategoryFormInputData>,
) -> Result<Redirect> {
    let base_url = state.base_url();
    let race_id = state
        .with_connection(move |conn| {
            diesel::update(special_categories::table.find(special_id.0))
                .set(data.0)
                .returning(special_categories::race_id)
                .get_result::<Id>(conn)
                .optional()
        })
        .await?
        .ok_or_else(|| {
            Error::NotFound(format!(
                "Special category with id {} not found",
                special_id.0
            ))
        })?;
    Ok(Redirect::to(&format!(
        "{base_url}/admin/races/{}/special_categories.html",
        race_id
    )))
}

#[axum::debug_handler(state = app_state::State)]
async fn add_special_category(
    state: AppState,
    race_id: Path<Id>,
    data: Form<SpecialCategoryFormInputData>,
) -> Result<Redirect> {
    let base_url = state.base_url();
    state
        .with_connection(move |conn| {
            diesel::insert_into(special_categories::table)
                .values((data.0, special_categories::race_id.eq(race_id.0)))
                .execute(conn)
        })
        .await?;
    Ok(Redirect::to(&format!(
        "{base_url}/admin/races/{}/special_categories.html",
        race_id.0
    )))
}
