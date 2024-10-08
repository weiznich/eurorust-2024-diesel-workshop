use super::Id;
use crate::database::schema::{
    competitions, participants, participants_in_special_category, races, special_categories,
};
use diesel::prelude::*;
use serde::Serialize;

#[derive(Queryable, Selectable, Serialize, Debug, Identifiable)]
#[diesel(table_name = competitions)]
pub struct Competition {
    pub id: Id,
    pub name: String,
    description: String,
    #[serde(serialize_with = "ymd_date")]
    date: time::Date,
    location: String,
    announcement: String,
}

#[derive(Queryable, Selectable, Serialize, Debug, Identifiable)]
#[diesel(table_name = participants)]
pub struct Participant {
    pub id: Id,
    last_name: String,
    first_name: String,
    club: Option<String>,
    category_id: Id,
}

#[derive(Queryable, Selectable, Serialize, Debug, Identifiable)]
pub struct Race {
    pub id: Id,
    pub name: String,
    competition_id: Id,
}

#[derive(Queryable, Selectable, Associations, Serialize, Debug, Identifiable)]
#[diesel(table_name = special_categories)]
#[diesel(belongs_to(crate::registration::RaceWithMinMaxAge, foreign_key = race_id))]
#[diesel(belongs_to(Race, foreign_key = race_id))]
pub struct SpecialCategories {
    pub id: Id,
    name: String,
    short_name: String,
    race_id: Id,
}

#[derive(Queryable, Selectable, Associations, Serialize, Debug, Identifiable)]
#[diesel(table_name = participants_in_special_category)]
#[diesel(primary_key(participant_id, special_category_id))]
#[diesel(belongs_to(crate::registration_list::ParticipantEntry, foreign_key = participant_id))]
pub struct SpecialCategoryPerParticipant {
    #[diesel(embed)]
    category: SpecialCategories,
    pub special_category_id: Id,
    participant_id: Id,
}

fn ymd_date<S>(d: &time::Date, ser: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    d.to_string().serialize(ser)
}
