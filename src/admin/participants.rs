//! Admin page setup for participants
use crate::app_state::{self, AppState};
use crate::database::schema::{
    categories, participants, participants_in_special_category, races, special_categories, starts,
};
use crate::database::Id;
use crate::errors::{Error, Result};
use crate::registration::{ParticipantForForm, ParticipantWithSpecialCategories, RegistrationForm};
use axum::extract::{Path, Query};
use axum::response::{Html, Redirect};
use axum::{Form, Router};
use diesel::expression::{is_aggregate, MixedAggregates, ValidGrouping};
use diesel::query_builder::QueryId;
use diesel::sql_types::Bool;
use diesel::sqlite::Sqlite;
use diesel::QueryDsl;
use diesel::{dsl, prelude::*};
use serde::{Deserialize, Serialize};

pub fn routes() -> Router<app_state::State> {
    let participants_routes = Router::new()
        .route(
            "/:participant_id/delete.html",
            axum::routing::get(delete_participant),
        )
        .route(
            "/:participant_id/edit.html",
            axum::routing::get(render_edit_participant),
        )
        .route("/:participant_id", axum::routing::post(update_participant))
        .route(
            "/add_participant.html",
            axum::routing::get(render_add_participant),
        );
    Router::new()
        .route(
            "/competitions/:competition_id/participants.html",
            axum::routing::get(list_participants_for_competition),
        )
        .route(
            "/competitions/:competition_id/add_participant",
            axum::routing::post(add_participant),
        )
        .route(
            "/races/:race_id/participants.html",
            axum::routing::get(list_participants_for_race),
        )
        .route(
            "/starts/:start_id/participants.html",
            axum::routing::get(list_participants_for_start),
        )
        .route(
            "/categories/:category_id/participants.html",
            axum::routing::get(list_participants_for_category),
        )
        .route(
            "/special_categories/:special_id/participants.html",
            axum::routing::get(list_participants_for_special_categories),
        )
        .nest("/participants", participants_routes)
}

#[derive(Serialize, Debug)]
pub struct Participant {
    pub id: Id,
    last_name: String,
    first_name: String,
    club: Option<String>,
    birth_year: i32,
    consent_agb: bool,
    category: String,
    race: String,
}

#[derive(Serialize)]
struct ParticipantListData {
    participants: Vec<Participant>,
    competition_id: Id,
    redirect_to: String,
    specifier: String,
}

#[axum::debug_handler(state = app_state::State)]
async fn list_participants_for_competition(
    state: AppState,
    comp_id: Path<Id>,
) -> Result<Html<String>> {
    let competition_name: String = todo!("Get the competition name here");

    list_participants_for_filter(
        state,
        races::competition_id.eq(comp_id.0),
        comp_id.0,
        format!("competitions/{}/participants.html", comp_id.0),
        competition_name,
    )
    .await
}

#[axum::debug_handler(state = app_state::State)]
async fn list_participants_for_race(state: AppState, race_id: Path<Id>) -> Result<Html<String>> {
    let (competition_id, race_name): (Id, String) = todo!("Load the competition_id and race_name");
    list_participants_for_filter(
        state,
        races::id.eq(race_id.0),
        competition_id,
        format!("races/{}/participants.html", race_id.0),
        race_name,
    )
    .await
}

#[axum::debug_handler(state = app_state::State)]
async fn list_participants_for_start(state: AppState, start_id: Path<Id>) -> Result<Html<String>> {
    let (competition_id, start_name): (Id, String) =
        todo!("Load the competition_id and the start name");

    list_participants_for_filter(
        state,
        starts::id.eq(start_id.0),
        competition_id,
        format!("starts/{}/participants.html", start_id.0),
        start_name,
    )
    .await
}

#[axum::debug_handler(state = app_state::State)]
async fn list_participants_for_category(
    state: AppState,
    category_id: Path<Id>,
) -> Result<Html<String>> {
    let (competition_id, race_name, category_label): (Id, String, String) =
        todo!("Load the competition id, race_name and category label");

    list_participants_for_filter(
        state,
        categories::id.eq(category_id.0),
        competition_id,
        format!("categories/{}/participants.html", category_id.0),
        format!("{category_label} ({race_name})"),
    )
    .await
}

#[axum::debug_handler(state = app_state::State)]
async fn list_participants_for_special_categories(
    state: AppState,
    special_id: Path<Id>,
) -> Result<Html<String>> {
    let (competition_id, special_label): (Id, String) =
        todo!("Load the competition id and the special category label");

    list_participants_for_filter(
        state,
        participants::id.eq_any(
            participants_in_special_category::table
                .inner_join(special_categories::table)
                .select(participants_in_special_category::participant_id)
                .filter(special_categories::id.eq(special_id.0)),
        ),
        competition_id,
        format!("special_categories/{}/participants.html", special_id.0),
        special_label,
    )
    .await
}

async fn list_participants_for_filter<F>(
    state: AppState,
    filter: F,
    competition_id: Id,
    redirect_to: String,
    specifier: String,
) -> Result<Html<String>>
where
    F: BoxableExpression<
            dsl::InnerJoinQuerySource<
                participants::table,
                dsl::InnerJoin<categories::table, dsl::InnerJoin<starts::table, races::table>>,
            >,
            Sqlite,
            SqlType = Bool,
        > + ValidGrouping<()>
        + QueryId
        + Send
        + 'static,
    F::IsAggregate: MixedAggregates<is_aggregate::No, Output = is_aggregate::No>,
{
    let participants: Vec<Participant> = todo!("Load participants");
    state.render_template(
        "admin_participant_list.html",
        ParticipantListData {
            participants,
            competition_id,
            redirect_to,
            specifier,
        },
    )
}

#[derive(Deserialize)]
struct RedirectInfo {
    redirect_to: String,
}

#[axum::debug_handler(state = app_state::State)]
async fn delete_participant(
    state: AppState,
    participant_id: Path<Id>,
    query: Query<RedirectInfo>,
) -> Result<Redirect> {
    let base_url = state.base_url();
    let count: usize = todo!("Delete participant");
    if count != 1 {
        Err(Error::NotFound(format!(
            "Participant with id {} not found",
            participant_id.0
        )))
    } else {
        Ok(Redirect::to(&format!(
            "{base_url}/admin/{}",
            query.redirect_to
        )))
    }
}

async fn load_participant_by_id(
    state: &AppState,
    participant_id: Id,
) -> Result<(ParticipantWithSpecialCategories, Id)> {
    todo!("Load participant with related data")
}

#[axum::debug_handler(state = app_state::State)]
async fn render_edit_participant(
    state: AppState,
    participant_id: Path<Id>,
) -> Result<Html<String>> {
    let (participant, competition_id) = load_participant_by_id(&state, participant_id.0).await?;
    crate::registration::render_registration_page_with_optional_data(
        state,
        competition_id,
        Some(participant),
        "edit_participant",
        format!("admin/participants/{}", participant_id.0),
    )
    .await
}

#[axum::debug_handler(state = app_state::State)]
async fn update_participant(
    state: AppState,
    participant_id: Path<Id>,
    query: Query<RedirectInfo>,
    data: Form<RegistrationForm>,
) -> Result<Redirect> {
    let base_url = state.base_url();
    let (_participant, competition_id) = load_participant_by_id(&state, participant_id.0).await?;
    data.0
        .into_database(&state, competition_id, Some(participant_id.0))
        .await?;
    Ok(Redirect::to(&format!(
        "{base_url}/admin/{}",
        query.redirect_to
    )))
}

#[axum::debug_handler(state = app_state::State)]
async fn render_add_participant(
    state: AppState,
    redirect: Query<RedirectInfo>,
) -> Result<Html<String>> {
    let mut participant = ParticipantWithSpecialCategories {
        participant: ParticipantForForm::default(),
        special_categories: Vec::new(),
    };
    let redirect_parts = redirect.redirect_to.split("/").collect::<Vec<_>>();
    let id = redirect_parts[1]
        .parse::<Id>()
        .map_err(|e| Error::InvalidInput(e.to_string()))?;
    let competition_id;
    match redirect_parts.as_slice() {
        ["competitions", _, _] => {
            competition_id = id;
        }
        ["races", _, _] => {
            participant.participant.race_id = Some(id);
            competition_id = todo!("Load the competition id based on the race_id");
        }
        ["starts", _, _] => {
            let (comp_id, race_id): (Id, Id) =
                todo!("Load competition and race id based on the start id");
            competition_id = comp_id;
            participant.participant.race_id = Some(race_id);
        }
        ["categories", _, _] => {
            participant.participant.category_id = Some(id);
            let (comp_id, race_id, from_age, male): (Id, Id, i32, bool) = todo!(
                "Load competition id, race_id, minimal age and gender based on the category id"
            );
            let year = time::OffsetDateTime::now_utc().year();
            competition_id = comp_id;
            participant.participant.race_id = Some(race_id);
            participant.participant.birth_year = Some(year - from_age);
            participant.participant.male = male;
        }
        ["special_categories", _, _] => {
            let (race_id, comp_id) =
                todo!("Load competition_id and race_id based on the special category id");
            participant.participant.race_id = Some(race_id);
            competition_id = comp_id;
            participant.special_categories.push(id);
        }
        _ => unreachable!(),
    }

    crate::registration::render_registration_page_with_optional_data(
        state,
        competition_id,
        Some(participant),
        "new_participant",
        format!(
            "admin/competitions/{}/add_participant?redirect_to={}",
            competition_id, redirect.redirect_to
        ),
    )
    .await
}

#[axum::debug_handler(state = app_state::State)]
async fn add_participant(
    state: AppState,
    competition_id: Path<Id>,
    redirect: Query<RedirectInfo>,
    form: Form<RegistrationForm>,
) -> Result<Redirect> {
    let base_url = state.base_url();
    form.0.into_database(&state, competition_id.0, None).await?;
    Ok(Redirect::to(&format!(
        "{base_url}/admin/{}",
        redirect.redirect_to
    )))
}
