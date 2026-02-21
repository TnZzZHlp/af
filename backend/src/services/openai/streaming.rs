use axum::{
    body::Body,
    http::{Response, header},
};
use futures_util::StreamExt;
use tokio::sync::oneshot;

use crate::error::{AppError, AppResult};

pub(super) fn build_streaming_response(
    upstream_response: reqwest::Response,
    status: reqwest::StatusCode,
    content_type: Option<header::HeaderValue>,
) -> AppResult<(Response<Body>, oneshot::Receiver<Vec<u8>>)> {
    let (response_body_tx, response_body_rx) = oneshot::channel::<Vec<u8>>();
    let mut stream_body = upstream_response.bytes_stream();
    let body_stream = async_stream::stream! {
        let mut captured = Vec::new();
        while let Some(next) = stream_body.next().await {
            match next {
                Ok(chunk) => {
                    captured.extend_from_slice(&chunk);
                    yield Ok::<_, std::convert::Infallible>(chunk);
                }
                Err(err) => {
                    tracing::error!(error = %err, "upstream stream error");
                    break;
                }
            }
        }
        let _ = response_body_tx.send(captured);
    };

    let mut builder = Response::builder().status(status);
    if let Some(value) = content_type {
        builder = builder.header(header::CONTENT_TYPE, value);
    }
    let response = builder
        .body(Body::from_stream(body_stream))
        .map_err(|err| AppError::Internal(err.into()))?;

    Ok((response, response_body_rx))
}

pub(super) fn build_buffered_response(
    status: reqwest::StatusCode,
    content_type: Option<header::HeaderValue>,
    body: Vec<u8>,
) -> AppResult<Response<Body>> {
    let mut builder = Response::builder().status(status);
    if let Some(value) = content_type {
        builder = builder.header(header::CONTENT_TYPE, value);
    }
    builder
        .body(Body::from(body))
        .map_err(|err| AppError::Internal(err.into()))
}
