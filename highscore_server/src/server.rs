use axum::{
	Router,
	body::Bytes,
	extract::State,
	http::{HeaderMap, HeaderValue, StatusCode},
	response::{IntoResponse, Response},
	routing::{get, post},
};
use std::{path::PathBuf, sync::Arc};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::{errors::HighscoreError, store::HighscoreStore};

pub struct HighscoreServer {
	store: HighscoreStore,
}

impl HighscoreServer {
	pub fn new(db_path: impl Into<PathBuf>) -> Self {
		Self {
			store: HighscoreStore::new(db_path),
		}
	}

	fn make_headers(content_type: &'static str) -> HeaderMap {
		let mut headers = HeaderMap::new();
		headers.insert("Content-Type", HeaderValue::from_static(content_type));
		headers
	}

	fn log_request(method: &str, path: &str) {
		let timestamp = OffsetDateTime::now_utc().format(&Rfc3339).unwrap_or_else(|_| String::from("0"));

		println!("[{timestamp:27}] {method:4} Request to \"{path}\"");
	}

	pub fn router(self: Arc<Self>) -> Router {
		Router::new()
			.route("/health", get(Self::handler_health))
			.route("/highscore", get(Self::handler_get))
			.route("/highscore", post(Self::handler_add))
			.with_state(self)
	}

	async fn handler_health() -> StatusCode {
		Self::log_request("GET", "/health");
		StatusCode::OK
	}

	async fn handler_get(State(server): State<Arc<Self>>) -> Response {
		Self::log_request("GET", "/highscore");
		match server.store.get_scores().await {
			Ok(ron_str) => (StatusCode::OK, Self::make_headers("application/x-ron"), ron_str).into_response(),
			Err(error) => error.into_response(),
		}
	}

	async fn handler_add(State(server): State<Arc<Self>>, body: Bytes) -> Response {
		Self::log_request("POST", "/highscore");
		const MAX_SIZE: usize = 5 * 1024; // 5KB limit
		if body.len() > MAX_SIZE {
			return HighscoreError::RequestTooLarge(MAX_SIZE).into_response();
		}

		let body_str = match String::from_utf8(body.to_vec()) {
			Ok(data) => data,
			Err(_) => return HighscoreError::InvalidUtf8.into_response(),
		};

		match server.store.add_score(body_str).await {
			Ok(()) => (StatusCode::OK, Self::make_headers("text/plain"), "ok").into_response(),
			Err(error) => error.into_response(),
		}
	}
}
