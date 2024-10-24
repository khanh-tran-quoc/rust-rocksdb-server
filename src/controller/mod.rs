use axum::{
    extract::{Query, State},
    response::Response,
};
use axum_macros::debug_handler;
use serde::Deserialize;

use crate::{response, service, AppState};

#[derive(Deserialize)]
pub struct PutQuery {
    key: String,
}

#[derive(Deserialize)]
pub struct GetQuery {
    key: String,
}

#[debug_handler]
pub async fn put(
    State(state): State<AppState>,
    Query(query): Query<PutQuery>,
    value: String,
) -> Response {
    let result = service::db::put(&state.rocksdb, &query.key, &value);
    match result {
        Ok(_) => {
            let message = format!("put key \"{}\" successfully", &query.key);
            response::success(message)
        }
        Err(e) => {
            let message = format!("cannot put key \"{}\": {}", &query.key, e);
            response::internal_server_error(message)
        }
    }
}

pub async fn get(State(state): State<AppState>, Query(query): Query<GetQuery>) -> Response {
    let result = service::db::get(&state.rocksdb, &query.key);
    match result {
        Ok(value) => match value {
            Some(value) => response::success(value),
            None => response::not_found(format!("key \"{}\" not found", &query.key)),
        },
        Err(e) => {
            let message = format!("cannot get key \"{}\": {}", &query.key, e);
            response::internal_server_error(message)
        }
    }
}
