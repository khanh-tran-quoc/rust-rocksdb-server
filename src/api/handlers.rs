use crate::{
    api::response,
    storage::rocksdb,
    telemetry::tracing::{current_span, extract_context_from_request},
    AppState,
};
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::Response,
};
use axum_macros::debug_handler;
use opentelemetry::trace::Span;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct PutQuery {
    key: String,
}

#[derive(Deserialize, Debug)]
pub struct GetQuery {
    key: String,
}

#[debug_handler]
pub async fn put(
    State(state): State<AppState>,
    Query(query): Query<PutQuery>,
    headers: HeaderMap,
    value: String,
) -> Response {
    let parent_cx = extract_context_from_request(&headers);
    let mut span = current_span(parent_cx);
    span.set_attribute(opentelemetry::KeyValue::new("key", query.key.clone()));

    let result = rocksdb::put(&state.rocksdb, &query.key, &value);
    match result {
        Ok(_) => {
            let message = format!("put key \"{}\" successfully", &query.key);
            span.set_status(opentelemetry::trace::Status::Ok);
            response::success(message)
        }
        Err(e) => {
            let message = format!("cannot put key \"{}\": {}", &query.key, e);
            span.set_status(opentelemetry::trace::Status::error(message.clone()));
            response::internal_server_error(message)
        }
    }
}

#[debug_handler]
pub async fn get(
    State(state): State<AppState>,
    Query(query): Query<GetQuery>,
    headers: HeaderMap,
) -> Response {
    let parent_cx = extract_context_from_request(&headers);
    let mut span = current_span(parent_cx);
    span.set_attribute(opentelemetry::KeyValue::new("key", query.key.clone()));

    let result = rocksdb::get(&state.rocksdb, &query.key);
    match result {
        Ok(value) => match value {
            Some(value) => {
                span.set_status(opentelemetry::trace::Status::Ok);
                response::success(value)
            }
            None => {
                let message = format!("key \"{}\" not found", &query.key);
                span.set_status(opentelemetry::trace::Status::error(message.clone()));
                response::not_found(message)
            }
        },
        Err(e) => {
            let message = format!("cannot get key \"{}\": {}", &query.key, e);
            span.set_status(opentelemetry::trace::Status::error(message.clone()));
            response::internal_server_error(message)
        }
    }
}
