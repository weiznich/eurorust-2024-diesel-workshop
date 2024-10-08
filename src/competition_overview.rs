//! A list of the various competitions in the database

use crate::app_state::AppState;
use crate::database::schema::competitions;
use crate::database::shared_models::Competition;
use crate::errors::Result;
use axum::response::Html;
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
use serde::Serialize;

#[derive(Serialize)]
struct CompetitionList {
    competitions: Vec<Competition>,
}

#[axum::debug_handler(state = crate::app_state::State)]
pub async fn render(state: AppState) -> Result<Html<String>> {
    let competitions = state
        .with_connection(move |conn| {
            competitions::table
                .select(Competition::as_select())
                .load(conn)
        })
        .await?;

    state.render_template("competition_list.html", CompetitionList { competitions })
}
