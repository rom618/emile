use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use emile::{ai::Claude, build_router, connect_db, AppState};
use tower::ServiceExt;

async fn app() -> (axum::Router, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("emile-test-{}-{}", std::process::id(), rand_suffix()));
    let db = connect_db(dir.join("t.db").to_str().unwrap()).await.unwrap();
    (build_router(AppState { db, claude: Claude::from_env() }, dir.to_str().unwrap()), dir)
}

fn rand_suffix() -> u128 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
}

async fn call(app: &axum::Router, method: &str, uri: &str, body: &str) -> (StatusCode, String) {
    let req = Request::builder().method(method).uri(uri).header("content-type", "application/json").body(Body::from(body.to_string())).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

#[tokio::test]
async fn add_then_list_orders_by_expiry() {
    let (app, dir) = app().await;
    let body = r#"{"source":"manual","items":[
        {"name":"rice","place":"cupboard","qty":1,"unit":"kg","cook_unit":null,"cook_per_unit":null,"expires_on":null},
        {"name":"egg","place":"fridge","qty":6,"unit":"piece","cook_unit":null,"cook_per_unit":null,"expires_on":"2026-10-20"},
        {"name":"potato","place":"cupboard","qty":500,"unit":"g","cook_unit":"piece","cook_per_unit":0.008,"expires_on":"2026-11-01"}]}"#;
    let (status, _) = call(&app, "POST", "/api/items", body).await;
    assert_eq!(status, StatusCode::OK);
    let (status, list) = call(&app, "GET", "/api/items", "").await;
    assert_eq!(status, StatusCode::OK);
    let names: Vec<String> = serde_json::from_str::<Vec<serde_json::Value>>(&list).unwrap().iter().map(|i| i["name"].as_str().unwrap().to_string()).collect();
    assert_eq!(names, ["egg", "potato", "rice"]);
    std::fs::remove_dir_all(dir).ok();
}

#[tokio::test]
async fn rejects_bad_place_and_source() {
    let (app, dir) = app().await;
    let bad_place = r#"{"source":"manual","items":[{"name":"x","place":"garage","qty":1,"unit":"g","cook_unit":null,"cook_per_unit":null,"expires_on":null}]}"#;
    assert_eq!(call(&app, "POST", "/api/items", bad_place).await.0, StatusCode::BAD_REQUEST);
    let bad_source = r#"{"source":"recipe","items":[]}"#;
    assert_eq!(call(&app, "POST", "/api/items", bad_source).await.0, StatusCode::BAD_REQUEST);
    std::fs::remove_dir_all(dir).ok();
}

#[tokio::test]
async fn parse_rejects_empty_text_without_calling_claude() {
    let (app, dir) = app().await;
    assert_eq!(call(&app, "POST", "/api/items/parse", r#"{"text":"  "}"#).await.0, StatusCode::BAD_REQUEST);
    std::fs::remove_dir_all(dir).ok();
}
