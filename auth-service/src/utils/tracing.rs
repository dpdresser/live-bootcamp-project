use std::time::Duration;

use axum::{body::Body, extract::Request, response::Response};
use tracing::{Level, Span};
use uuid::Uuid;

pub fn init_tracing() {
    tracing_subscriber::fmt()
        .compact()
        .with_max_level(tracing::Level::DEBUG)
        .init();
}

// Creates a new tacing span with a unique request ID for each incoming request.
// This helps in tracking and correlating logs for individual requests.
pub fn make_span_with_request_id(request: &Request<Body>) -> Span {
    let request_id = Uuid::new_v4();
    tracing::span!(
        Level::INFO,
        "[REQUEST]",
        method = tracing::field::display(request.method()),
        uri = tracing::field::display(request.uri()),
        version = tracing::field::display(format!("{:?}", request.version())),
        request_id = tracing::field::display(request_id),
    )
}

// Logs an event indicating the start of a request.
pub fn on_request(_request: &Request<Body>, _span: &Span) {
    tracing::event!(Level::INFO, "[REQUEST START]");
}

// Logs an event indicating the end of a request, including it's latency and status code.
// If the status code indicates an error, logs at ERROR level.
pub fn on_response(response: &Response, latency: Duration, _span: &Span) {
    let status = response.status();
    let status_code = status.as_u16();
    let status_code_class = status_code / 100;

    match status_code_class {
        4..=5 => {
            tracing::event!(
                Level::ERROR,
                latency = ?latency,
                status = status_code,
                "[REQUEST END]"
            )
        }
        _ => {
            tracing::event!(
                Level::INFO,
                latency = ?latency,
                status = status_code,
                "[REQUEST END]"
            )
        }
    };
}
