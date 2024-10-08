use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use race_timing::service_config::Config;
use std::path::PathBuf;
use tower::ServiceExt;

mod competition_list;
mod registration;

// provide a server config suitable for testing
//
// the `test_data` argument indicates whether we want to include test data
// on startup or not
fn test_config(test_data: bool) -> Config {
    let template_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");

    Config {
        port: 8000,
        address: "127.0.0.1".parse().unwrap(),
        database_url: ":memory:".into(),
        insert_test_data: test_data,
        base_url: "".into(),
        template_dir,
        is_test: true,
    }
}

#[tokio::test]
async fn translations_work() {
    let (router, _state) = race_timing::setup(test_config(false)).await;

    // requesting english translation works
    let resp = router
        .clone()
        .oneshot(
            Request::get("/index.html")
                .header("Accept-Language", "en")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let data = resp.into_body().collect().await.unwrap().to_bytes();
    let string = String::from_utf8(data.to_vec()).unwrap();
    assert!(string.contains("Competitions"));

    // requesting german translation works
    let resp = router
        .clone()
        .oneshot(
            Request::get("/index.html")
                .header("Accept-Language", "de")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let data = resp.into_body().collect().await.unwrap().to_bytes();
    let string = String::from_utf8(data.to_vec()).unwrap();
    assert!(string.contains("Wettkämpfe"));

    // default is english
    let resp = router
        .clone()
        .oneshot(Request::get("/index.html").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let data = resp.into_body().collect().await.unwrap().to_bytes();
    let string = String::from_utf8(data.to_vec()).unwrap();
    assert!(string.contains("Competitions"));

    // non existing language falls back to english
    let resp = router
        .clone()
        .oneshot(
            Request::get("/index.html")
                .header("Accept-Language", "fr; sr")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let data = resp.into_body().collect().await.unwrap().to_bytes();
    let string = String::from_utf8(data.to_vec()).unwrap();
    assert!(string.contains("Competitions"));

    // fallback chain works
    let resp = router
        .clone()
        .oneshot(
            Request::get("/index.html")
                .header("Accept-Language", "fr,de;q=0.8")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let data = resp.into_body().collect().await.unwrap().to_bytes();
    let string = String::from_utf8(data.to_vec()).unwrap();
    assert!(string.contains("Wettkämpfe"), "{string}");
}
