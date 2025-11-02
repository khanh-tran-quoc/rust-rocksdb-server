use axum::{extract::DefaultBodyLimit, routing::post, Router};
use h_rocksdb::{api::handlers, AppState};
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace as sdktrace, Resource};
use rocksdb::DB;
use std::{env, process, sync::Arc};
use tokio::runtime::Builder;

fn init_tracer() -> Result<sdktrace::SdkTracerProvider, sdktrace::TraceError> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://127.0.0.1:43318".to_string());
    let traces_endpoint = format!("{}/v1/traces", endpoint.trim_end_matches('/'));

    let service_name =
        env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "rust-rocksdb-server".to_string());

    let resource = Resource::builder()
        .with_service_name(service_name)
        .with_attribute(KeyValue::new("service.version", env!("CARGO_PKG_VERSION")))
        .build();

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_endpoint(traces_endpoint)
        .with_protocol(Protocol::HttpBinary)
        .build()
        .map_err(|err| err.to_string())?;

    let provider = sdktrace::SdkTracerProvider::builder()
        .with_resource(resource)
        .with_sampler(sdktrace::Sampler::ParentBased(Box::new(
            sdktrace::Sampler::AlwaysOn,
        )))
        .with_batch_exporter(exporter)
        .build();

    let _ = global::set_tracer_provider(provider.clone());

    Ok(provider)
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
        let tracer_provider = match init_tracer() {
            Ok(provider) => Some(provider),
            Err(err) => {
                eprintln!("Failed to initialize OpenTelemetry tracing: {err}");
                None
            }
        };

        let app = Router::new()
            .route("/put", post(handlers::put))
            .route("/get", post(handlers::get))
            .layer(DefaultBodyLimit::max(200000000))
            .with_state(state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:4000")
            .await
            .unwrap();

        println!("Server running",);
        if let Err(err) = axum::serve(listener, app).await {
            eprintln!("Server error: {err}");
        }
        if let Some(provider) = tracer_provider {
            if let Err(err) = provider.shutdown() {
                eprintln!("Failed to flush OpenTelemetry spans: {err}");
            }
        }
    });
}
