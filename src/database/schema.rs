// @generated automatically by Diesel CLI.

diesel::table! {
    categories (id) {
        id -> Integer,
        label -> Text,
        from_age -> Integer,
        to_age -> Integer,
        male -> Bool,
        start_id -> Integer,
    }
}

diesel::table! {
    competitions (id) {
        id -> Integer,
        name -> Text,
        description -> Text,
        date -> Date,
        location -> Text,
        announcement -> Text,
    }
}

diesel::table! {
    participants (id) {
        id -> Integer,
        last_name -> Text,
        first_name -> Text,
        club -> Nullable<Text>,
        category_id -> Integer,
        consent_agb -> Bool,
        birth_year -> Integer,
    }
}

diesel::table! {
    participants_in_special_category (participant_id, special_category_id) {
        participant_id -> Integer,
        special_category_id -> Integer,
    }
}

diesel::table! {
    races (id) {
        id -> Integer,
        name -> Text,
        competition_id -> Integer,
    }
}

diesel::table! {
    session_records (id) {
        id -> Binary,
        data -> Text,
        expiry_date -> TimestamptzSqlite,
    }
}

diesel::table! {
    special_categories (id) {
        id -> Integer,
        short_name -> Text,
        name -> Text,
        race_id -> Integer,
    }
}

diesel::table! {
    starts (id) {
        id -> Integer,
        name -> Text,
        time -> Timestamp,
        race_id -> Integer,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        name -> Text,
        password -> Text,
    }
}

diesel::joinable!(categories -> starts (start_id));
diesel::joinable!(participants -> categories (category_id));
diesel::joinable!(participants_in_special_category -> participants (participant_id));
diesel::joinable!(participants_in_special_category -> special_categories (special_category_id));
diesel::joinable!(races -> competitions (competition_id));
diesel::joinable!(special_categories -> races (race_id));
diesel::joinable!(starts -> races (race_id));

diesel::allow_tables_to_appear_in_same_query!(
    categories,
    competitions,
    participants,
    participants_in_special_category,
    races,
    session_records,
    special_categories,
    starts,
    users,
);
