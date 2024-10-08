//! Render a list of all participants for a specific competition grouped by races
use crate::app_state::{self, AppState};
use crate::database::schema::{
    categories, competitions, participants, races, special_categories, starts,
};
use crate::database::shared_models::{
    Competition, Race, SpecialCategories, SpecialCategoryPerParticipant,
};
use crate::database::Id;
use crate::errors::{Error, Result};
use axum::extract::Path;
use axum::response::Html;
use axum::Router;
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use serde::Serialize;
use time::PrimitiveDateTime;

pub fn routes() -> Router<app_state::State> {
    Router::new().route(
        "/:event_id/registration_list.html",
        axum::routing::get(render_registration_list),
    )
}

/// Data for a specific participants
#[derive(Queryable, Selectable, Debug, serde::Serialize, Identifiable)]
#[diesel(table_name = participants)]
#[diesel(check_for_backend(Sqlite))]
pub struct ParticipantEntry {
    /// id of the participant
    #[serde(skip)]
    id: Id,
    /// first name of the participant
    first_name: String,
    /// last name of the participant
    last_name: String,
    /// club of the participant
    club: Option<String>,
    /// birth year of the participant
    birth_year: i32,
    /// start time for this participant
    #[diesel(select_expression = starts::time)]
    start_time: PrimitiveDateTime,
    /// category label for this participant
    #[diesel(select_expression = categories::label)]
    class: String,
    /// name of the race the participant participantes in
    #[serde(skip)]
    #[diesel(select_expression = races::name)]
    race_name: String,
}

#[derive(Debug, serde::Serialize)]
struct ParticipantEntryWithSpecialCategory {
    /// inner participant data
    #[serde(flatten)]
    participant: ParticipantEntry,
    /// a list of flags whether a participant is part of a special category or not
    /// the order of this list is expected to match the order of ParticipantsPerRace::special_categories
    special_categories: Vec<bool>,
}

#[derive(Debug, serde::Serialize)]
struct ParticipantsPerRace {
    /// Name of the race
    race_name: String,
    /// A list of participants for this race ordered by age
    participants: Vec<ParticipantEntryWithSpecialCategory>,
    /// A list of special categories for this race
    special_categories: Vec<SpecialCategories>,
}

/// Data used to render the participant list
///
/// See `templates/registration_list.html` for the relevant template
#[derive(Serialize)]
struct RegistrationListData {
    /// race specific participant data
    race_map: Vec<ParticipantsPerRace>,
    /// general information about the competition
    competition_info: Competition,
}

#[axum::debug_handler(state = app_state::State)]
async fn render_registration_list(state: AppState, event_id: Path<Id>) -> Result<Html<String>> {
    let event_id = event_id.0;

    let (
        participant_list,
        competition_info,
        races,
        special_categories,
        special_categories_per_participant,
    ) = state
        .with_connection(move |conn| {
            let competition_info = competitions::table
                .find(event_id)
                .select(Competition::as_select())
                .first(conn)
                .optional()?;
            let races = races::table
                .inner_join(starts::table.inner_join(categories::table))
                .order_by((categories::from_age, races::name))
                .filter(races::competition_id.eq(event_id))
                .group_by(races::id)
                .select(Race::as_select())
                .load(conn)?;

            let special_categories = SpecialCategories::belonging_to(&races)
                .select(SpecialCategories::as_select())
                .load(conn)?;
            let special_categories = special_categories.grouped_by(&races);

            let participants = participants::table
                .inner_join(categories::table.inner_join(starts::table.inner_join(races::table)))
                .filter(races::competition_id.eq(event_id))
                .order_by((
                    categories::from_age,
                    races::name,
                    participants::birth_year.desc(),
                    participants::first_name,
                    participants::last_name,
                ))
                .select(ParticipantEntry::as_select())
                .load(conn)?;

            let special_categories_per_participant =
                SpecialCategoryPerParticipant::belonging_to(&participants)
                    .inner_join(special_categories::table)
                    .select(SpecialCategoryPerParticipant::as_select())
                    .load(conn)?;

            let special_categories_per_participant =
                special_categories_per_participant.grouped_by(&participants);

            Ok((
                participants,
                competition_info,
                races,
                special_categories,
                special_categories_per_participant,
            ))
        })
        .await?;
    let competition_info = competition_info
        .ok_or_else(|| Error::NotFound(format!("No competition for id {} found", event_id)))?;

    let mut participant_iter = participant_list
        .into_iter()
        .zip(special_categories_per_participant)
        .peekable();

    let race_map = races
        .into_iter()
        .zip(special_categories)
        .map(|(race, special_categories)| {
            let mut participants = Vec::new();
            while let Some((p, _special_categories_per_participant)) = participant_iter.peek() {
                if *p.race_name == race.name {
                    let (p, special_categories_per_participant) =
                        participant_iter.next().expect("We peeked");

                    let special_categories = special_categories
                        .iter()
                        .map(|cat| {
                            special_categories_per_participant
                                .iter()
                                .any(|c| c.special_category_id == cat.id)
                        })
                        .collect();
                    participants.push(ParticipantEntryWithSpecialCategory {
                        participant: p,
                        special_categories,
                    });
                } else {
                    break;
                }
            }
            ParticipantsPerRace {
                race_name: race.name,
                participants,
                special_categories,
            }
        })
        .collect::<Vec<_>>();

    state.render_template(
        "registration_list.html",
        RegistrationListData {
            race_map,
            competition_info,
        },
    )
}
