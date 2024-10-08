//! Routes for handling the registration of a new participant
use crate::app_state::{self, AppState};
use crate::database::schema::{categories, participants, races};
use crate::database::shared_models::{Competition, Race, SpecialCategories};
use crate::database::Id;
use crate::errors::{Error, Result};
use axum::extract::Path;
use axum::response::{Html, Redirect};
use axum::Router;
use diesel::associations::HasTable;
use diesel::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

pub fn routes() -> Router<app_state::State> {
    Router::new()
        .route(
            "/:event_id/registration.html",
            axum::routing::get(render_registration_page),
        )
        .route(
            "/:event_id/participant/",
            axum::routing::post(add_participant),
        )
}

/// Existing participant data used for the update form via
/// the admin pages
#[derive(Queryable, Selectable, Serialize, Debug, Default)]
#[diesel(table_name = participants)]
pub struct ParticipantForForm {
    last_name: String,
    first_name: String,
    club: Option<String>,
    #[diesel(select_expression = participants::birth_year.nullable())]
    pub birth_year: Option<i32>,
    #[diesel(select_expression = categories::male)]
    pub male: bool,
    #[diesel(select_expression = participants::category_id.nullable())]
    pub category_id: Option<Id>,
    #[diesel(select_expression = races::id.nullable())]
    pub race_id: Option<Id>,
    consent_agb: bool,
}

/// Existing participant data used for the update form via the admin
/// pages
#[derive(Serialize)]
pub struct ParticipantWithSpecialCategories {
    #[serde(flatten)]
    pub participant: ParticipantForForm,
    pub special_categories: Vec<Id>,
}

/// Data used to render the participant form
///
/// see `templates/registration.html` for the template
#[derive(Serialize)]
struct RegistrationPageData {
    /// Which competition is the form for
    event: Competition,
    /// Which races exist for the competition
    race_data: Vec<RaceWithSpecialCategory>,
    /// minimal age of participants
    min_age: Option<i32>,
    /// maximal age of participant
    max_age: Option<i32>,
    /// Title displayed in the HTML head tag
    head_title: String,
    /// Title displayed as headline on the page
    title: String,
    /// optinal participant data for updating an existing participant
    ///
    /// This is used to prefill the form
    ///
    /// That can be used from the admin pages
    participant: Option<ParticipantWithSpecialCategories>,
    /// The target uri the form posts data to
    /// `base_url` is automatically prepended by the template
    target_uri: String,
}

/// Data for a specific race with minimal and maximal age for this race
#[derive(Queryable, serde::Serialize)]
pub struct RaceWithMinMaxAge {
    race: Race,
    min_age: i32,
    max_age: i32,
}

/// Data for a specific race including special categories for this race
#[derive(serde::Serialize)]
struct RaceWithSpecialCategory {
    #[serde(flatten)]
    race: RaceWithMinMaxAge,
    special_categories: Vec<SpecialCategories>,
}

impl HasTable for RaceWithMinMaxAge {
    type Table = races::table;
    fn table() -> Self::Table {
        races::table
    }
}
impl<'ident> Identifiable for &'ident RaceWithMinMaxAge {
    type Id = &'ident Id;
    fn id(self) -> Self::Id {
        &self.race.id
    }
}
impl<'ident> Identifiable for &'_ &'ident RaceWithMinMaxAge {
    type Id = &'ident Id;
    fn id(self) -> Self::Id {
        &self.race.id
    }
}

/// Data returned from the registration form
#[derive(Debug, serde::Deserialize)]
pub struct RegistrationForm {
    /// The race the participant is registerd for
    pub race: Id,
    /// Whether or not the participant is male
    #[serde(default)]
    pub male: bool,
    /// More data about the participant
    #[serde(flatten)]
    pub new_participant: NewParticipant,
    /// For which special categories the participant registered for
    #[serde(flatten)]
    pub special_categories: HashMap<Id, String>,
}

#[derive(Debug, serde::Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = participants)]
pub struct NewParticipant {
    /// last name of the new/updated participants
    #[diesel(column_name = "last_name")]
    pub lastname: String,
    /// first name of the new/updated participants
    #[diesel(column_name = "first_name")]
    pub firstname: String,
    /// club the new/updated participants is registerd for
    pub club: String,
    /// whether or not the participants has consent to our AGB
    #[diesel(column_name = "consent_agb")]
    #[serde(deserialize_with = "parse_checkbox")]
    pub consent: bool,
    /// The birth year of the participant
    #[diesel(column_name = "birth_year")]
    #[serde(deserialize_with = "parse_string")]
    pub age: i32,
}

fn parse_checkbox<'de, D>(d: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s = <&str>::deserialize(d)?;
    Ok(s == "on")
}

fn parse_string<'de, D>(d: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = <&str>::deserialize(d)?;
    s.parse().map_err(serde::de::Error::custom)
}

impl RegistrationForm {
    /// Are the provided registration form data valid
    fn is_valid(&self) -> Result<()> {
        if !self.new_participant.consent {
            tracing::debug!(?self);
            Err(Error::InvalidInput(String::from(
                "Expect that you consent to the \
                 participant conditions",
            )))
        } else {
            Ok(())
        }
    }

    /// Insert the registration form data into the database
    ///
    /// If a `participant_id` is provided we need to handle an update
    /// otherwise it's an insert of existing data
    pub async fn into_database(
        self,
        state: &AppState,
        competition_id: Id,
        participant_id: Option<Id>,
    ) -> Result<()> {
        self.is_valid()?;
        let age = time::OffsetDateTime::now_utc().year() - self.new_participant.age;
        let special_categories_id = self.special_categories.keys().copied().collect::<Vec<_>>();

        // for inserting/updating participant data we need to perform several database related operations
        //
        // (Consider focusing on inserting first and adding update support later)
        //
        // 1. Get all relevant data:
        //    + Resolve Race id + birth year to relevat category
        //    + Resolve special categories by id (verify that they exist)
        // 2. Insert participant
        // 3. Insert special category mapping

        todo!("Insert the new participant into the database")
    }
}

/// Load data relevant for the registration form for a certain competition
fn load_competition_data(
    conn: &mut SqliteConnection,
    path: Id,
) -> QueryResult<Option<(Competition, Vec<RaceWithSpecialCategory>)>> {
    // to render the registration page we need to have various information
    //
    // 1. Competition information
    // 2. Information about possible races, including category related data + special categories
    todo!("Load the relevant competition data")
}

#[axum::debug_handler(state = app_state::State)]
async fn render_registration_page(state: AppState, event_id: Path<Id>) -> Result<Html<String>> {
    render_registration_page_with_optional_data(
        state,
        event_id.0,
        None,
        "registration",
        format!("{}/participant/", event_id.0),
    )
    .await
}

/// Render the registration page
///
/// This is used from the ordinary public registration page and from the
/// edit/add participant entries in the admin section
pub async fn render_registration_page_with_optional_data(
    state: AppState,
    event_id: Id,
    participant: Option<ParticipantWithSpecialCategories>,
    title: &str,
    target_uri: String,
) -> Result<Html<String>> {
    let (competition, races): (Competition, Vec<RaceWithSpecialCategory>) =
        todo!("Load required data to render registration page");

    let min_age = races.iter().map(|r| r.race.min_age).max();
    let max_age = races.iter().map(|r| r.race.max_age).min();
    let params = HashMap::from([("competition", &competition.name as &str)]);
    state.render_template(
        "registration.html",
        RegistrationPageData {
            race_data: races,
            min_age,
            max_age,
            participant,
            head_title: state.translation(&format!("short_{title}")),
            title: state.translation_with_params(title, params),
            event: competition,
            target_uri,
        },
    )
}

/// Handle adding a new participant
#[axum::debug_handler(state = app_state::State)]
async fn add_participant(
    state: AppState,
    event_id: Path<Id>,
    form_data: axum::extract::Form<RegistrationForm>,
) -> Result<Redirect> {
    form_data.0.into_database(&state, event_id.0, None).await?;
    let base_url = state.base_url();
    Ok(Redirect::to(&format!(
        "{base_url}/{}/registration_list.html",
        event_id.0
    )))
}
