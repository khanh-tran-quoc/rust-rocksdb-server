use axum::http::HeaderMap;
use opentelemetry::{
    global::{self, BoxedSpan},
    trace::Tracer,
    Context,
};
use opentelemetry_http::HeaderExtractor;

pub fn extract_context_from_request(header: &HeaderMap) -> Context {
    global::get_text_map_propagator(|propagator| propagator.extract(&HeaderExtractor(header)))
}

pub fn current_span(parent_cx: Context) -> BoxedSpan {
    let tracer = global::tracer("rocksdb");
    tracer
        .span_builder("put")
        .with_kind(opentelemetry::trace::SpanKind::Server)
        .start_with_context(&tracer, &parent_cx)
}
