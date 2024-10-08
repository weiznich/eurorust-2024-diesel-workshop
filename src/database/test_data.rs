use std::collections::HashMap;

use argon2::password_hash::SaltString;
use argon2::Argon2;
use argon2::PasswordHasher;
use time::Date;

use crate::database::schema::{
    categories, competitions, participants, participants_in_special_category, races,
    special_categories, starts, users,
};
use crate::database::Id;
use diesel::prelude::*;

#[derive(diesel::Insertable, Clone)]
#[diesel(table_name = categories)]
pub(crate) struct NewCategory {
    pub(crate) label: String,
    pub(crate) from_age: i32,
    pub(crate) to_age: i32,
    pub(crate) male: bool,
    pub(crate) start_id: Id,
}

impl NewCategory {
    pub(crate) fn new(
        label: &'static str,
        from_age: i32,
        to_age: i32,
        male: bool,
        start_id: Id,
    ) -> Self {
        Self {
            label: label.to_owned(),
            from_age,
            to_age,
            male,
            start_id,
        }
    }

    pub(crate) fn clone_for_femal(input: impl IntoIterator<Item = Self>) -> Vec<Self> {
        input
            .into_iter()
            .flat_map(|i| {
                [
                    i.clone(),
                    Self {
                        male: false,
                        label: i.label.replace(" m", " f").replace("M ", "W "),
                        ..i.clone()
                    },
                ]
            })
            .collect()
    }

    pub(crate) fn clone_for_start(
        input: impl IntoIterator<Item = Self>,
        start_id: Id,
    ) -> Vec<Self> {
        input.into_iter().map(|i| Self { start_id, ..i }).collect()
    }
}

/// create a set of test data to see something in the application
pub(crate) fn insert_test_data(conn: &mut SqliteConnection) -> QueryResult<()> {
    conn.transaction(|conn| {
        let competition_count = competitions::table.count().get_result::<i64>(conn)?;
        if competition_count != 0 {
            panic!("Database already contains test data, cannot insert new test data");
        }
        let competition_id = diesel::insert_into(competitions::table)
            .values((
                competitions::name.eq("Country Cross Race Vienna 2024"),
                competitions::description.eq("An example competition for EuroRust in Vienna"),
                competitions::date
                    .eq(Date::from_calendar_date(2024, time::Month::October, 9).expect("We know that this is a valid date")),
                competitions::location.eq("Vienna"),
                competitions::announcement.eq("https://example.com/announcement.htlm"),
            ))
            .returning(competitions::id)
            .get_result::<Id>(conn)?;

        let inserted_races = diesel::insert_into(races::table)
            .values(vec![
                (
                    races::name.eq("400m"),
                    races::competition_id.eq(competition_id),
                ),
                (
                    races::name.eq("800m"),
                    races::competition_id.eq(competition_id),
                ),
                (
                    races::name.eq("1200m"),
                    races::competition_id.eq(competition_id),
                ),
                (
                    races::name.eq("5,5km Nordic Walking"),
                    races::competition_id.eq(competition_id),
                ),
                (
                    races::name.eq("2,5km"),
                    races::competition_id.eq(competition_id),
                ),
                (
                    races::name.eq("11km"),
                    races::competition_id.eq(competition_id),
                ),
                (
                    races::name.eq("5,5km"),
                    races::competition_id.eq(competition_id),
                ),
            ])
            .execute(conn)?;
        let race_map = races::table
            .order_by(races::id.desc())
            .limit(inserted_races as i64)
            .select((races::name, races::id))
            .load_iter(conn)?
            .collect::<QueryResult<HashMap<String, Id>>>()?;

        let inserted_starts = diesel::insert_into(starts::table)
            .values(vec![
                (
                    starts::name.eq("400m"),
                    starts::time.eq(time::macros::datetime!(2024-10-09 10:00:00)),
                    starts::race_id.eq(race_map["400m"]),
                ),
                (
                    starts::name.eq("800m"),
                    starts::time.eq(time::macros::datetime!(2024-10-09 10:10:00)),
                    starts::race_id.eq(race_map["800m"]),
                ),
                (
                    starts::name.eq("1200m"),
                    starts::time.eq(time::macros::datetime!(2024-10-09 10:20:00)),
                    starts::race_id.eq(race_map["1200m"]),
                ),
                (
                    starts::name.eq("5,5km Nordic Walking"),
                    starts::time.eq(time::macros::datetime!(2024-10-09 10:25:00)),
                    starts::race_id.eq(race_map["5,5km Nordic Walking"]),
                ),
                (
                    starts::name.eq("2,5km"),
                    starts::time.eq(time::macros::datetime!(2024-10-09 10:30:00)),
                    starts::race_id.eq(race_map["2,5km"]),
                ),
                (
                    starts::name.eq("11km"),
                    starts::time.eq(time::macros::datetime!(2024-10-09 10:50:00)),
                    starts::race_id.eq(race_map["11km"]),
                ),
                (
                    starts::name.eq("5,5km (Femal)"),
                    starts::time.eq(time::macros::datetime!(2024-10-09 11:00:00)),
                    starts::race_id.eq(race_map["5,5km"]),
                ),
                (
                    starts::name.eq("5,5km (Male)"),
                    starts::time.eq(time::macros::datetime!(2024-10-09 11:10:00)),
                    starts::race_id.eq(race_map["5,5km"]),
                ),
            ])
            .execute(conn)?;
        let start_map = starts::table
            .order_by(starts::id.desc())
            .limit(inserted_starts as i64)
            .select((starts::name, starts::id))
            .load_iter(conn)?
            .collect::<QueryResult<HashMap<String, Id>>>()?;

        let categories_400m = NewCategory::clone_for_femal([
            NewCategory::new("U4 m", 0, 3, true, start_map["400m"]),
            NewCategory::new("U6 m", 4, 5, true, start_map["400m"]),
        ]);
        let categories_800m =
            NewCategory::clone_for_femal([NewCategory::new("U8 m", 6, 7, true, start_map["800m"])]);
        let categories_1200m = NewCategory::clone_for_femal([NewCategory::new(
            "U10 m",
            8,
            9,
            true,
            start_map["800m"],
        )]);

        let categories_5500m_nw = vec![
            NewCategory::new("Men", 0, 99, true, start_map["5,5km Nordic Walking"]),
            NewCategory::new("Woman", 0, 99, false, start_map["5,5km Nordic Walking"]),
        ];

        let categories_2500m = NewCategory::clone_for_femal([NewCategory::new(
            "U12 m",
            10,
            11,
            true,
            start_map["2,5km"],
        )]);

        let u14_16 = vec![
            NewCategory::new("U14 m", 12, 13, true, start_map["5,5km (Male)"]),
            NewCategory::new("U16 m", 14, 15, true, start_map["5,5km (Male)"]),
        ];

        let u18 = NewCategory::new("U18 m", 16, 17, true, start_map["11km"]);
        let jun_m = NewCategory::new("U20 m", 18, 19, true, start_map["11km"]);
        let jun_w = NewCategory::new("U20 f", 18, 19, false, start_map["11km"]);

        let categories_er = vec![
            NewCategory::new("M 21", 20, 29, true, start_map["11km"]),
            NewCategory::new("M 31", 30, 39, true, start_map["11km"]),
            NewCategory::new("M 41", 40, 49, true, start_map["11km"]),
            NewCategory::new("M 51", 50, 59, true, start_map["11km"]),
            NewCategory::new("M 61", 60, 69, true, start_map["11km"]),
            NewCategory::new("M 71", 70, 99, true, start_map["11km"]),
        ];

        let mut categories_11000m =
            NewCategory::clone_for_femal(categories_er.clone().into_iter().chain([u18.clone()]));
        categories_11000m.push(jun_m.clone());
        categories_11000m.push(jun_w.clone());

        let mut categories_5500m_m = NewCategory::clone_for_start(
            categories_er.clone().into_iter().chain(u14_16).chain([u18]),
            start_map["5,5km (Male)"],
        );

        let mut categories_5500m_w = NewCategory::clone_for_start(
            NewCategory::clone_for_femal(categories_5500m_m.clone()),
            start_map["5,5km (Femal)"],
        );
        categories_5500m_w.extend(NewCategory::clone_for_start(
            [jun_w],
            start_map["5,5km (Femal)"],
        ));
        categories_5500m_m.extend(NewCategory::clone_for_start(
            [jun_m],
            start_map["5,5km (Male)"],
        ));

        diesel::insert_into(categories::table)
            .values(
                categories_400m
                    .into_iter()
                    .chain(categories_800m)
                    .chain(categories_1200m)
                    .chain(categories_5500m_nw)
                    .chain(categories_2500m)
                    .chain(categories_11000m)
                    .chain(categories_5500m_w)
                    .chain(categories_5500m_m)
                    .collect::<Vec<_>>(),
            )
            .execute(conn)?;

        let special_category_id = diesel::insert_into(special_categories::table)
            .values((
                special_categories::name.eq("Fastest Person over 11km"),
                special_categories::short_name.eq("FP"),
                special_categories::race_id.eq(race_map["11km"]),
            ))
            .returning(special_categories::id)
            .get_result::<Id>(conn)?;

        let cat_11km = categories::table.filter(categories::label.eq("M 21").and(categories::start_id.eq(start_map["11km"]))).select(categories::id).first::<Id>(conn)?;
        let cat_5km = categories::table.filter(categories::label.eq("W 21").and(categories::start_id.eq(start_map["5,5km (Femal)"]))).select(categories::id).first::<Id>(conn)?;

        diesel::insert_into(participants::table)
            .values([
                (
                    participants::last_name.eq("Doe"),
                    participants::first_name.eq("John"),
                    participants::club.eq("Eurorust Running Club"),
                    participants::category_id.eq(cat_11km),
                    participants::consent_agb.eq(true),
                    participants::birth_year.eq(1995),
                ),
                (
                    participants::last_name.eq("Doe"),
                    participants::first_name.eq("Jane"),
                    participants::club.eq("Eurorust Running Club"),
                    participants::category_id.eq(cat_5km),
                    participants::consent_agb.eq(true),
                    participants::birth_year.eq(1995),
                )
            ]).execute(conn)?;

        let johns_id = participants::table.filter(participants::first_name.eq("John")).select(participants::id).first::<Id>(conn)?;

        diesel::insert_into(participants_in_special_category::table)
            .values((
                participants_in_special_category::participant_id.eq(johns_id),
                participants_in_special_category::special_category_id.eq(special_category_id)
            ))
            .execute(conn)?;

        let password = "admin";
        let salt = SaltString::generate(&mut rand::rngs::OsRng);
        let password_hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .expect("We know that we can hash this password");

        println!("Created user `admin` with password `admin`, go to /admin/login.html to access the admin area");
        diesel::insert_into(users::table)
            .values((
                users::name.eq("admin"),
                users::password.eq(password_hash.to_string()),
            ))
            .execute(conn)
    })?;
    Ok(())
}
