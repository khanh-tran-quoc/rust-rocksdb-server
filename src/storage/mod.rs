//! Storage Layer
//!
//! This module handles all database operations and persistence:
//! - RocksDB operations (get/put)
//! - Future: caching, transactions, batch operations

pub mod rocksdb;
