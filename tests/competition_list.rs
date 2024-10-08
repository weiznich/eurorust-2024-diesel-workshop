use axum::body::Body;
use axum::http::{Request, StatusCode};
use diesel::prelude::*;
use diesel::ExpressionMethods;
use http_body_util::BodyExt;
use race_timing::database::schema::competitions;
use race_timing::database::Id;
use time::Date;
use tower::ServiceExt;

#[tokio::test]
async fn list_competitions_contains_all_competition() {
    let (router, state) = race_timing::setup(super::test_config(false)).await;

    let res = router
        .clone()
        .oneshot(Request::get("/index.html").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let content =
        String::from_utf8(res.into_body().collect().await.unwrap().to_bytes().to_vec()).unwrap();
    let li_count = content.matches("<li>").count();
    assert_eq!(li_count, 0);

    let competitions = state
        .with_connection(|conn| {
            let date = Date::from_calendar_date(2024, time::Month::October, 9).unwrap();
            diesel::insert_into(competitions::table)
                .values([
                    (
                        competitions::name.eq("Test Competition 1"),
                        competitions::description.eq("Desc 1"),
                        competitions::date.eq(date),
                        competitions::location.eq("Somewhere 1"),
                        competitions::announcement.eq("http://example.com/1"),
                    ),
                    (
                        competitions::name.eq("Test Competition 2"),
                        competitions::description.eq("Desc 2"),
                        competitions::date.eq(date),
                        competitions::location.eq("Somewhere 2"),
                        competitions::announcement.eq("http://example.com/2"),
                    ),
                ])
                .execute(conn)?;
            competitions::table
                .select((competitions::id, competitions::name))
                .load::<(Id, String)>(conn)
        })
        .await
        .unwrap();

    let res = router
        .oneshot(Request::get("/index.html").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let content =
        String::from_utf8(res.into_body().collect().await.unwrap().to_bytes().to_vec()).unwrap();
    let li_count = content.matches("<li>").count();
    assert_eq!(li_count, 2);

    // we search for <a href="{url}">{title}</a> here
    let parts = content
        .split("<a href=\"")
        .skip(1)
        .flat_map(|s| {
            let (first, _ignore) = s.split_once("</a>")?;
            let (url, name) = first.split_once("\">")?;
            Some((url.trim(), name.trim()))
        })
        .collect::<Vec<_>>();
    assert_eq!(parts.len(), 2);
    for ((competition_id, expected_title), (url, real_title)) in competitions.into_iter().zip(parts)
    {
        assert_eq!(format!("/{competition_id}/registration_list.html"), url);
        assert_eq!(expected_title, real_title);
    }
}
