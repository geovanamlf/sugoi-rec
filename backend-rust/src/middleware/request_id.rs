use std::time::Instant;

use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

pub const X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

const MAX_REQUEST_ID_LENGTH: usize = 128;

pub async fn add_request_id(mut request: Request, next: Next) -> Response {
    let started_at = Instant::now();

    let request_id = extract_request_id(&request).unwrap_or_else(generate_request_id);

    let method = request.method().clone();
    let path = request.uri().path().to_string();

    request.extensions_mut().insert(request_id.clone());

    tracing::info!(
        request_id = %request_id,
        method = %method,
        path = %path,
        "HTTP request started"
    );

    let mut response = next.run(request).await;

    if let Ok(header_value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert(X_REQUEST_ID, header_value);
    }

    let status = response.status();
    let elapsed_ms = started_at.elapsed().as_millis();

    tracing::info!(
        request_id = %request_id,
        method = %method,
        path = %path,
        status = status.as_u16(),
        elapsed_ms = elapsed_ms,
        "HTTP request completed"
    );

    response
}

fn extract_request_id(request: &Request) -> Option<String> {
    let value = request.headers().get(&X_REQUEST_ID)?;
    let value = value.to_str().ok()?.trim();

    if value.is_empty() || value.len() > MAX_REQUEST_ID_LENGTH {
        return None;
    }

    if !value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return None;
    }

    Some(value.to_string())
}

fn generate_request_id() -> String {
    Uuid::new_v4().to_string()
}
