//! Admin page setup for special_categories
use crate::app_state::{self, AppState};
use crate::database::Id;
use crate::errors::Result;
use axum::extract::Path;
use axum::response::{Html, Redirect};
use axum::{Form, Router};
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

#[derive(Serialize)]
struct SpecialCategoryData {
    id: Id,
    short_name: String,
    name: String,
    participant_count: i64,
}

#[derive(Serialize)]
struct ListSpecialCategoriesData {
    special_categories: Vec<SpecialCategoryData>,
    race_id: Id,
}

#[axum::debug_handler(state = app_state::State)]
async fn list_special_categories(state: AppState, race_id: Path<Id>) -> Result<Html<String>> {
    let special_categories: Vec<SpecialCategoryData> =
        todo!("Load all special categories for the provided race");
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
    let race_id: Id = todo!("Delete the special category and load the corresponding race id");
    Ok(Redirect::to(&format!(
        "{base_url}/admin/races/{}/special_categories.html",
        race_id
    )))
}

#[derive(Serialize)]
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
    todo!("Check if the race exists");
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
    let special_category: EditSpecialCategoriesData =
        todo!("Load relevant data to render the edit special category form");
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

#[derive(Deserialize)]
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
    let race_id: Id = todo!("Update the special category and get the race id");
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
    todo!("Insert a new special category");
    Ok(Redirect::to(&format!(
        "{base_url}/admin/races/{}/special_categories.html",
        race_id.0
    )))
}
