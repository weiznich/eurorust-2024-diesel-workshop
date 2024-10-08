//! Admin page setup for categories
use crate::app_state::{self, AppState};
use crate::database::schema::{categories, participants, starts};
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

#[derive(Serialize, Selectable, Queryable)]
#[diesel(table_name = categories)]
#[diesel(check_for_backend(Sqlite))]
struct CategoryData {
    id: Id,
    label: String,
    from_age: i32,
    to_age: i32,
    male: bool,
    #[diesel(select_expression = dsl::count(participants::id.nullable()))]
    participant_count: i64,
}

#[axum::debug_handler(state = app_state::State)]
async fn list_categories_per_start(state: AppState, start_id: Path<Id>) -> Result<Html<String>> {
    let categories = state
        .with_connection(move |conn| {
            categories::table
                .filter(categories::start_id.eq(start_id.0))
                .left_join(participants::table)
                .group_by(categories::id)
                .select(CategoryData::as_select())
                .order_by(categories::from_age)
                .load(conn)
        })
        .await?;

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
    let start_id = state
        .with_connection(move |conn| {
            diesel::delete(categories::table.find(category_id.0))
                .returning(categories::start_id)
                .get_result::<Id>(conn)
                .optional()
        })
        .await?
        .ok_or_else(|| Error::NotFound(format!("No category with id {} found", category_id.0)))?;

    Ok(Redirect::to(&format!(
        "{base_url}/admin/starts/{}/categories.html",
        start_id
    )))
}

#[derive(Serialize, Queryable, Selectable)]
#[diesel(table_name = categories)]
#[diesel(check_for_backend(Sqlite))]
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

#[derive(Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = categories)]
struct CategoryFormInputData {
    label: String,
    from_age: i32,
    to_age: i32,
    male: bool,
}

#[axum::debug_handler(state = app_state::State)]
async fn render_create_category(state: AppState, start_id: Path<Id>) -> Result<Html<String>> {
    state
        .with_connection(move |conn| {
            starts::table
                .find(start_id.0)
                .select(starts::id)
                .get_result::<Id>(conn)
                .optional()
        })
        .await?
        .ok_or_else(|| Error::NotFound(format!("Start with id {} not found", start_id.0)))?;

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
    state
        .with_connection(move |conn| {
            diesel::insert_into(categories::table)
                .values((data.0, categories::start_id.eq(start_id.0)))
                .execute(conn)
        })
        .await?;

    Ok(Redirect::to(&format!(
        "{base_url}/admin/starts/{}/categories.html",
        start_id.0
    )))
}

#[axum::debug_handler(state = app_state::State)]
async fn render_edit_category(state: AppState, category_id: Path<Id>) -> Result<Html<String>> {
    let category = state
        .with_connection(move |conn| {
            categories::table
                .find(category_id.0)
                .select(EditCategoryData::as_select())
                .first(conn)
                .optional()
        })
        .await?
        .ok_or_else(|| Error::NotFound(format!("Category with id {} not found", category_id.0)))?;

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
    let start_id = state
        .with_connection(move |conn| {
            diesel::update(categories::table.find(category_id.0))
                .set(data.0)
                .returning(categories::start_id)
                .get_result::<Id>(conn)
                .optional()
        })
        .await?
        .ok_or_else(|| Error::NotFound(format!("Category with id {} not found", category_id.0)))?;
    Ok(Redirect::to(&format!(
        "{base_url}/admin/starts/{}/categories.html",
        start_id
    )))
}
