use std::fmt::Debug;

use axum::response::{IntoResponse, Response};
use axum::{http::StatusCode, Json};
use serde::Serialize;

pub fn success<T: Serialize>(body: T) -> Response {
    (StatusCode::OK, Json(body)).into_response()
}

pub fn not_found<T: Serialize>(body: T) -> Response {
    (StatusCode::NOT_FOUND, Json(body)).into_response()
}

pub fn internal_server_error<T: Serialize + Debug>(body: T) -> Response {
    println!("{:#?}", body);
    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}
