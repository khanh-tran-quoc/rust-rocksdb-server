use axum::extract::DefaultBodyLimit;
use axum::routing::post;
use axum::Router;
use opentelemetry::KeyValue;
use opentelemetry_otlp::ExportConfig;
use opentelemetry_otlp::Protocol;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::metrics::reader::DefaultTemporalitySelector;
use opentelemetry_sdk::trace::RandomIdGenerator;
use opentelemetry_sdk::trace::Sampler;
use opentelemetry_sdk::Resource;
use rocksdb::DB;
use std::env;
use std::process;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Builder;

use opentelemetry::global::ObjectSafeSpan;
use opentelemetry::trace::{SpanKind, Status};
use opentelemetry::{global, trace::Tracer};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::trace::{self};
use opentelemetry_stdout::SpanExporter;

use once_cell::sync::Lazy;
mod controller;
mod response;
mod service;

fn init_tracer() {
    global::set_text_map_propagator(TraceContextPropagator::new());

    // Setup tracerprovider with stdout exporter
    // that prints the spans to stdout.
    // let provider = TracerProvider::builder()
    //     .with_simple_exporter(SpanExporter::default())
    //     .build();

    // global::set_tracer_provider(provider);

    let tracer_provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://127.0.0.1:43317/v1/traces")
                .with_protocol(Protocol::Grpc)
                .with_timeout(Duration::from_secs(3)), // .with_metadata(map),
        )
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::AlwaysOn)
                .with_id_generator(RandomIdGenerator::default())
                .with_max_events_per_span(64)
                .with_max_attributes_per_span(16)
                .with_max_events_per_span(16)
                .with_resource(Resource::new(vec![KeyValue::new(
                    "service.name",
                    "rocksdb",
                )])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .unwrap();
    global::set_tracer_provider(tracer_provider);
    global::tracer("rocksdb");
}

fn get_db_path() -> String {
    match env::current_dir() {
        Ok(path) => {
            if let Some(str_path) = path.join("rocks.db").to_str() {
                return str_path.to_string();
            } else {
                eprintln!("Failed to convert path to string.");
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error getting current directory: {}", e);
            process::exit(1);
        }
    }
}

#[derive(Clone, Debug)]
pub struct AppState {
    rocksdb: Arc<DB>,
}

fn main() {
    let rocksdb_path = get_db_path();
    let db = DB::open_default(rocksdb_path).unwrap();

    let runtime = Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    let state = AppState {
        rocksdb: Arc::new(db),
    };

    runtime.block_on(async {
        init_tracer();

        let app = Router::new()
            .route("/put", post(controller::put))
            .route("/get", post(controller::get))
            .layer(DefaultBodyLimit::max(200000000))
            .with_state(state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:4000")
            .await
            .unwrap();

        println!("Server running",);
        axum::serve(listener, app).await.unwrap();
    });
}
