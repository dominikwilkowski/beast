use axum::{
	Router,
	body::Bytes,
	extract::State,
	http::{HeaderMap, HeaderValue, StatusCode},
	routing::{get, post},
};
use std::{path::PathBuf, sync::Arc};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::store::HighscoreStore;

pub struct HighscoreServer {
	store: HighscoreStore,
}

impl HighscoreServer {
	pub fn new(db_path: impl Into<PathBuf>) -> Self {
		Self {
			store: HighscoreStore::new(db_path),
		}
	}

	pub async fn get_highscore_handler(&self) -> (StatusCode, HeaderMap, String) {
		match self.store.get_scores().await {
			Ok(ron_str) => {
				let mut headers = HeaderMap::new();
				headers.insert("Content-Type", HeaderValue::from_static("application/ron"));

				(StatusCode::OK, headers, ron_str)
			},
			Err(error) => {
				let mut headers = HeaderMap::new();
				headers.insert("Content-Type", HeaderValue::from_static("text/plain"));

				(StatusCode::INTERNAL_SERVER_ERROR, headers, error)
			},
		}
	}

	pub async fn add_highscore_handler(&self, body: Bytes) -> (StatusCode, HeaderMap, String) {
		let mut headers = HeaderMap::new();
		headers.insert("Content-Type", HeaderValue::from_static("text/plain"));

		// 5KB limit
		if body.len() > 1024 * 5 {
			return (StatusCode::BAD_REQUEST, headers, String::from("Request body too large"));
		}

		let body_str = match String::from_utf8(body.to_vec()) {
			Ok(data) => data,
			Err(_) => return (StatusCode::BAD_REQUEST, headers, String::from("Invalid UTF-8 in request body")),
		};

		match self.store.add_score(body_str).await {
			Ok(()) => (StatusCode::OK, headers, String::from("ok")),
			Err(error) => {
				if error.contains("Name cannot be empty") || error.contains("RON parsing error") {
					(StatusCode::BAD_REQUEST, headers, error)
				} else {
					(StatusCode::INTERNAL_SERVER_ERROR, headers, error)
				}
			},
		}
	}

	pub fn router(self: Arc<Self>) -> Router {
		Router::new()
			.route("/health", post(Self::handler_health))
			.route("/get-highscore", get(Self::handler_get))
			.route("/add-highscore", post(Self::handler_add))
			.with_state(self)
	}

	async fn handler_health() -> StatusCode {
		println!(
			"[{}] GET  Request /health",
			OffsetDateTime::now_utc().format(&Rfc3339).unwrap_or(String::from("(error formatting timestamp)"))
		);
		StatusCode::OK
	}

	async fn handler_get(State(server): State<Arc<Self>>) -> (StatusCode, HeaderMap, String) {
		println!(
			"[{}] GET  Request /get-highscore",
			OffsetDateTime::now_utc().format(&Rfc3339).unwrap_or(String::from("(error formatting timestamp)"))
		);
		server.get_highscore_handler().await
	}

	async fn handler_add(State(server): State<Arc<Self>>, body: Bytes) -> (StatusCode, HeaderMap, String) {
		println!(
			"[{}] POST Request /set-highscore",
			OffsetDateTime::now_utc().format(&Rfc3339).unwrap_or(String::from("(error formatting timestamp)"))
		);
		server.add_highscore_handler(body).await
	}
}
