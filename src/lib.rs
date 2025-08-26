use rocksdb::DB;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct AppState {
    pub rocksdb: Arc<DB>,
}

/// API layer - HTTP handlers and response utilities
pub mod api;

/// Storage layer - Database operations
pub mod storage;

/// Telemetry layer - Observability and monitoring
pub mod telemetry;
