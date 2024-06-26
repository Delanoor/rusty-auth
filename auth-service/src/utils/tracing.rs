use std::time::Duration;

use axum::{body::Body, extract::Request, response::Response};
use color_eyre::eyre::Result;
use tracing::{Level, Span};
use tracing_error::ErrorLayer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

pub fn init_tracing() -> Result<()> {
    let fmt_layer = fmt::layer().pretty();

    // Create a filter layer to control the verbosity of logs
    // Try to get the filter config from the environment variables
    // if fails, default to "info" log level

    let filter_layer: EnvFilter =
        EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("sqlx=debug,info"))?;

    // Build the tracing subscriber registry with the formatting layer, filter layer,
    // and the error layer for enhanced error reporting

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();

    Ok(())
}

// Creates a new tracing span with a unique request ID for each incoming request.
pub fn make_span_with_request_id(request: &Request<Body>) -> Span {
    let request_id = uuid::Uuid::new_v4();
    tracing::span!(
        Level::INFO,
        "[REQUEST]",
        method = tracing::field::display(request.method()),
        uri = tracing::field::display(request.uri()),
        version = tracing::field::debug(request.version()),
        request_id = tracing::field::display(request_id),
    )
}

// Logs an event indicating the start of a request.
pub fn on_request(_request: &Request<Body>, _span: &Span) {
    tracing::event!(Level::INFO, "[REQUEST START]");
}

// Logs an event indicating the end of a request, including its latency and status code.
// If error (4xx, 5xx), it logs at the ERROR level
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
