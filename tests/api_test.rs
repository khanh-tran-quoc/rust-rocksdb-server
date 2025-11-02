use axum::{
    body::{to_bytes, Body},
    extract::DefaultBodyLimit,
    http::{Request, StatusCode},
    routing::post,
    Router,
};
use h_rocksdb::{api::handlers, AppState};
use rocksdb::{Options, DB};
use std::sync::Arc;
use tempfile::TempDir;
use tower::util::ServiceExt;

fn create_test_app() -> (Router, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let path = temp_dir.path();

    let mut opts = Options::default();
    opts.create_if_missing(true);
    let db = DB::open(&opts, path).expect("Failed to open test database");

    let state = AppState {
        rocksdb: Arc::new(db),
    };

    let app = Router::new()
        .route("/put", post(handlers::put))
        .route("/get", post(handlers::get))
        .layer(DefaultBodyLimit::max(200000000))
        .with_state(state);

    (app, temp_dir)
}

#[tokio::test]
async fn test_put_endpoint_success() {
    let (app, _temp_dir) = create_test_app();

    let request = Request::builder()
        .method("POST")
        .uri("/put?key=test_key")
        .header("content-type", "text/plain")
        .body(Body::from("test_value"))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("successfully"));
}

#[tokio::test]
async fn test_put_endpoint_missing_key() {
    let (app, _temp_dir) = create_test_app();

    let request = Request::builder()
        .method("POST")
        .uri("/put")
        .header("content-type", "text/plain")
        .body(Body::from("test_value"))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_endpoint_existing_key() {
    let (app, _temp_dir) = create_test_app();

    // First, put a value
    let put_request = Request::builder()
        .method("POST")
        .uri("/put?key=test_key")
        .header("content-type", "text/plain")
        .body(Body::from("test_value"))
        .unwrap();

    let put_response = app.clone().oneshot(put_request).await.unwrap();
    assert_eq!(put_response.status(), StatusCode::OK);

    // Then, get the value
    let get_request = Request::builder()
        .method("POST")
        .uri("/get?key=test_key")
        .body(Body::empty())
        .unwrap();

    let get_response = app.oneshot(get_request).await.unwrap();
    assert_eq!(get_response.status(), StatusCode::OK);

    let body = to_bytes(get_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("test_value"));
}

#[tokio::test]
async fn test_get_endpoint_non_existing_key() {
    let (app, _temp_dir) = create_test_app();

    let request = Request::builder()
        .method("POST")
        .uri("/get?key=non_existing_key")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("not found"));
}

#[tokio::test]
async fn test_get_endpoint_missing_key_param() {
    let (app, _temp_dir) = create_test_app();

    let request = Request::builder()
        .method("POST")
        .uri("/get")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_put_empty_value() {
    let (app, _temp_dir) = create_test_app();

    let put_request = Request::builder()
        .method("POST")
        .uri("/put?key=empty_key")
        .header("content-type", "text/plain")
        .body(Body::from(""))
        .unwrap();

    let put_response = app.clone().oneshot(put_request).await.unwrap();
    assert_eq!(put_response.status(), StatusCode::OK);

    // Verify we can get the empty value back
    let get_request = Request::builder()
        .method("POST")
        .uri("/get?key=empty_key")
        .body(Body::empty())
        .unwrap();

    let get_response = app.oneshot(get_request).await.unwrap();
    assert_eq!(get_response.status(), StatusCode::OK);

    let body = to_bytes(get_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert_eq!(body_str, "\"\"");
}

#[tokio::test]
async fn test_put_overwrite_value() {
    let (app, _temp_dir) = create_test_app();

    // Put initial value
    let put_request1 = Request::builder()
        .method("POST")
        .uri("/put?key=overwrite_key")
        .header("content-type", "text/plain")
        .body(Body::from("initial_value"))
        .unwrap();

    let put_response1 = app.clone().oneshot(put_request1).await.unwrap();
    assert_eq!(put_response1.status(), StatusCode::OK);

    // Overwrite with new value
    let put_request2 = Request::builder()
        .method("POST")
        .uri("/put?key=overwrite_key")
        .header("content-type", "text/plain")
        .body(Body::from("new_value"))
        .unwrap();

    let put_response2 = app.clone().oneshot(put_request2).await.unwrap();
    assert_eq!(put_response2.status(), StatusCode::OK);

    // Verify the new value
    let get_request = Request::builder()
        .method("POST")
        .uri("/get?key=overwrite_key")
        .body(Body::empty())
        .unwrap();

    let get_response = app.oneshot(get_request).await.unwrap();
    assert_eq!(get_response.status(), StatusCode::OK);

    let body = to_bytes(get_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("new_value"));
}

#[tokio::test]
async fn test_unicode_key_and_value() {
    let (app, _temp_dir) = create_test_app();

    // Use URL encoding for Unicode key
    let encoded_key = urlencoding::encode("unicode_test_key");

    // Put unicode value
    let put_request = Request::builder()
        .method("POST")
        .uri(format!("/put?key={}", encoded_key))
        .header("content-type", "text/plain; charset=utf-8")
        .body(Body::from("unicode_test_value"))
        .unwrap();

    let put_response = app.clone().oneshot(put_request).await.unwrap();
    assert_eq!(put_response.status(), StatusCode::OK);

    // Get unicode value
    let get_request = Request::builder()
        .method("POST")
        .uri(format!("/get?key={}", encoded_key))
        .body(Body::empty())
        .unwrap();

    let get_response = app.oneshot(get_request).await.unwrap();
    assert_eq!(get_response.status(), StatusCode::OK);

    let body = to_bytes(get_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("unicode_test_value"));
}

#[tokio::test]
async fn test_large_value() {
    let (app, _temp_dir) = create_test_app();

    let large_value = "x".repeat(1024 * 1024); // 1MB

    let put_request = Request::builder()
        .method("POST")
        .uri("/put?key=large_key")
        .header("content-type", "text/plain")
        .body(Body::from(large_value.clone()))
        .unwrap();

    let put_response = app.clone().oneshot(put_request).await.unwrap();
    assert_eq!(put_response.status(), StatusCode::OK);

    // Verify we can retrieve the large value
    let get_request = Request::builder()
        .method("POST")
        .uri("/get?key=large_key")
        .body(Body::empty())
        .unwrap();

    let get_response = app.oneshot(get_request).await.unwrap();
    assert_eq!(get_response.status(), StatusCode::OK);

    let body = to_bytes(get_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert_eq!(body_str.trim_matches('"').len(), large_value.len());
}

#[tokio::test]
async fn test_multiple_operations() {
    let (app, _temp_dir) = create_test_app();

    let test_data = vec![("key1", "value1"), ("key2", "value2"), ("key3", "value3")];

    // Put all values
    for (key, value) in &test_data {
        let put_request = Request::builder()
            .method("POST")
            .uri(format!("/put?key={}", key))
            .header("content-type", "text/plain")
            .body(Body::from(*value))
            .unwrap();

        let put_response = app.clone().oneshot(put_request).await.unwrap();
        assert_eq!(put_response.status(), StatusCode::OK);
    }

    // Get all values
    for (key, expected_value) in &test_data {
        let get_request = Request::builder()
            .method("POST")
            .uri(format!("/get?key={}", key))
            .body(Body::empty())
            .unwrap();

        let get_response = app.clone().oneshot(get_request).await.unwrap();
        assert_eq!(get_response.status(), StatusCode::OK);

        let body = to_bytes(get_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains(expected_value));
    }
}
