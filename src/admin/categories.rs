//! Admin page setup for categories
use crate::app_state::{self, AppState};
use crate::database::Id;
use crate::errors::Result;
use axum::extract::Path;
use axum::response::{Html, Redirect};
use axum::{Form, Router};
use serde::{Deserialize, Serialize};

pub(crate) fn routes() -> Router<app_state::State> {
    let categories_router = Router::new()
        .route(
            "/:category_id/delete.html",
            axum::routing::get(delete_category),
        )
        .route(
            "/:category_id/edit.html",
            axum::routing::get(render_edit_category),
        )
        .route("/:category_id", axum::routing::post(update_category));

    Router::new()
        .nest("/categories/", categories_router)
        .route(
            "/starts/:start_id/categories.html",
            axum::routing::get(list_categories_per_start),
        )
        .route(
            "/starts/:start_id/create_category.html",
            axum::routing::get(render_create_category),
        )
        .route(
            "/starts/:start_id/create_category",
            axum::routing::post(create_category),
        )
}

#[derive(Serialize)]
struct ListCategoiesData {
    start_id: Id,
    categories: Vec<CategoryData>,
}

#[derive(Serialize)]
struct CategoryData {
    id: Id,
    label: String,
    from_age: i32,
    to_age: i32,
    male: bool,
    participant_count: i64,
}

#[axum::debug_handler(state = app_state::State)]
async fn list_categories_per_start(state: AppState, start_id: Path<Id>) -> Result<Html<String>> {
    let categories = todo!("list categories for start here");

    state.render_template(
        "admin_category_list.html",
        ListCategoiesData {
            start_id: start_id.0,
            categories,
        },
    )
}

#[axum::debug_handler(state = app_state::State)]
async fn delete_category(state: AppState, category_id: Path<Id>) -> Result<Redirect> {
    let base_url = state.base_url();
    let start_id: Id = todo!();

    Ok(Redirect::to(&format!(
        "{base_url}/admin/starts/{}/categories.html",
        start_id
    )))
}

#[derive(Serialize)]
struct EditCategoryData {
    label: String,
    from_age: i32,
    to_age: i32,
    male: bool,
    start_id: Id,
}

#[derive(Serialize)]
struct CategoryFormData {
    start_id: Id,
    category: Option<EditCategoryData>,
    target_url: String,
    title: String,
}

#[derive(Deserialize)]
struct CategoryFormInputData {
    label: String,
    from_age: i32,
    to_age: i32,
    male: bool,
}

#[axum::debug_handler(state = app_state::State)]
async fn render_create_category(state: AppState, start_id: Path<Id>) -> Result<Html<String>> {
    todo!("Load start here to verify it exists");

    state.render_template(
        "edit_category.html",
        CategoryFormData {
            start_id: start_id.0,
            category: None,
            target_url: format!("starts/{}/create_category", start_id.0),
            title: state.translation("new_category"),
        },
    )
}

#[axum::debug_handler(state = app_state::State)]
async fn create_category(
    state: AppState,
    start_id: Path<Id>,
    data: Form<CategoryFormInputData>,
) -> Result<Redirect> {
    let base_url = state.base_url();
    todo!("Insert data here");

    Ok(Redirect::to(&format!(
        "{base_url}/admin/starts/{}/categories.html",
        start_id.0
    )))
}

#[axum::debug_handler(state = app_state::State)]
async fn render_edit_category(state: AppState, category_id: Path<Id>) -> Result<Html<String>> {
    let category: EditCategoryData = todo!("Get category data here");

    state.render_template(
        "edit_category.html",
        CategoryFormData {
            start_id: category.start_id,
            category: Some(category),
            target_url: format!("categories/{}", category_id.0),
            title: state.translation("edit_category"),
        },
    )
}

#[axum::debug_handler(state = app_state::State)]
async fn update_category(
    state: AppState,
    category_id: Path<Id>,
    data: Form<CategoryFormInputData>,
) -> Result<Redirect> {
    let base_url = state.base_url();
    let start_id: Id = todo!("Verify that start exists here");
    Ok(Redirect::to(&format!(
        "{base_url}/admin/starts/{}/categories.html",
        start_id
    )))
}
