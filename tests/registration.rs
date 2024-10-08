use axum::body::Body;
use axum::http::{Request, StatusCode};
use diesel::prelude::*;
use diesel::ExpressionMethods;
use race_timing::database::schema::{categories, competitions, participants, races, starts};
use race_timing::database::Id;
use tower::ServiceExt;

#[tokio::test]
async fn check_registration() {
    let (router, state) = race_timing::setup(super::test_config(true)).await;

    let (participant_count, competition_id, km11_id) = state
        .with_connection(|conn| {
            diesel::delete(participants::table).execute(conn)?;
            let km11_id = races::table
                .filter(races::name.eq("11km"))
                .select(races::id)
                .get_result::<Id>(conn)?;
            let count = participants::table.count().get_result::<i64>(conn)?;
            let competition_id = competitions::table
                .select(competitions::id)
                .first::<Id>(conn)?;
            Ok((count, competition_id, km11_id))
        })
        .await
        .unwrap();

    assert_eq!(participant_count, 0);

    let value = serde_json::json! {{
        "lastname": "Doe",
        "firstname": "John",
        "age": 1996,
        "male": true,
        "club": "Eurorust Running Club",
        "race": km11_id.to_string(),
        "consent": "on",
    }};
    let body = serde_urlencoded::to_string(&value).unwrap();
    let resp = router
        .oneshot(
            Request::post(format!("/{competition_id}/participant/"))
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::new(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::SEE_OTHER);

    state
        .with_connection(move |conn| {
            let count = participants::table.count().get_result::<i64>(conn)?;
            assert_eq!(count, 1);
            let (_id, last_name, first_name, club, category_id, conset_agb, birth_year) =
                participants::table
                    .first::<(Id, String, String, Option<String>, Id, bool, i32)>(conn)?;

            assert_eq!(last_name, "Doe");
            assert_eq!(first_name, "John");
            assert_eq!(club, Some("Eurorust Running Club".into()));
            assert_eq!(conset_agb, true);
            assert_eq!(birth_year, 1996);
            let cat_id = categories::table
                .inner_join(starts::table)
                .filter(starts::race_id.eq(km11_id))
                .filter(categories::label.eq("M 21"))
                .select(categories::id)
                .first::<Id>(conn)?;
            assert_eq!(cat_id, category_id);
            Ok(())
        })
        .await
        .unwrap();
}
