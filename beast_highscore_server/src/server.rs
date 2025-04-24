//! this module contains the server itself listening to a port and handing respondses

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

const MAX_SIZE: usize = 5 * 1024; // 5KB limit

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

		if std::env::var("DEBUG").unwrap_or(String::from("false")) == "true" {
			std::thread::sleep(std::time::Duration::from_secs(5));
		}

		StatusCode::OK
	}

	async fn handler_get(State(server): State<Arc<Self>>) -> Response {
		Self::log_request("GET", "/highscore");

		if std::env::var("DEBUG").unwrap_or(String::from("false")) == "true" {
			std::thread::sleep(std::time::Duration::from_secs(5));
		}

		match server.store.get_scores().await {
			Ok(ron_str) => (StatusCode::OK, Self::make_headers("application/x-ron"), ron_str).into_response(),
			Err(error) => error.into_response(),
		}
	}

	async fn handler_add(State(server): State<Arc<Self>>, body: Bytes) -> Response {
		Self::log_request("POST", "/highscore");

		if std::env::var("DEBUG").unwrap_or(String::from("false")) == "true" {
			std::thread::sleep(std::time::Duration::from_secs(5));
		}

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

#[cfg(test)]
mod tests {
	use super::*;
	use axum::{
		body::{Body, to_bytes},
		http::{Method, Request, StatusCode},
	};
	use beast_common::Highscores;
	use beast_common::levels::Level;
	use ron::de::from_str;
	use std::sync::Arc;
	use tower::util::ServiceExt;

	use crate::common::TempFile;

	#[tokio::test]
	async fn health_check_test() {
		let temp_file = TempFile::new(".temp_file_server_1.ron", None);
		let server = Arc::new(HighscoreServer::new(&temp_file.path));
		let app = server.router();

		let request = Request::builder().uri("/health").body(Body::empty()).unwrap();
		let response = app.oneshot(request).await.unwrap();

		assert_eq!(response.status(), StatusCode::OK, "The health check should be OK");
	}

	#[tokio::test]
	async fn post_get_highscore_test() {
		let temp_file = TempFile::new(".temp_file_server_2.ron", None);
		let server = Arc::new(HighscoreServer::new(&temp_file.path));
		let app = server.router();

		// get highscore from empty file
		let request = Request::builder().uri("/highscore").body(Body::empty()).unwrap();
		let response = app.clone().oneshot(request).await.unwrap();
		assert_eq!(response.status(), StatusCode::OK, "The health check should be OK");
		let body_bytes = to_bytes(response.into_body(), 2024).await.unwrap();
		let body_str = std::str::from_utf8(&body_bytes).unwrap();
		assert_eq!(body_str, r#"(scores:[])"#, "The highscore should be empty");

		// post a new highscore
		let ron_payload = r#"(name: "Dom", score: 5, level: One)"#;
		let request = Request::builder()
			.method(Method::POST)
			.uri("/highscore")
			.header("content-type", "application/x-ron")
			.body(Body::from(ron_payload))
			.unwrap();
		let response = app.clone().oneshot(request).await.unwrap();
		assert_eq!(response.status(), StatusCode::OK, "The post request should be successful");

		// get the highscore and make sure it contains the new score
		let request = Request::builder().uri("/highscore").body(Body::empty()).unwrap();
		let response = app.oneshot(request).await.unwrap();
		assert_eq!(response.status(), StatusCode::OK, "The get request should be successful");
		let body_bytes = to_bytes(response.into_body(), 2024).await.unwrap();
		let body_str = std::str::from_utf8(&body_bytes).unwrap();
		let scores = from_str::<Highscores>(body_str).unwrap();
		assert_eq!(scores.scores.len(), 1, "The highscore should contain one score in total");
		assert_eq!(scores.scores[0].name, "Dom", "The top highscore name should be what we posted earlier");
		assert_eq!(scores.scores[0].score, 5, "The top highscore score should be what we posted earlier");
		assert_eq!(scores.scores[0].level, Level::One, "The top highscore level should be what we posted earlier");
	}

	#[tokio::test]
	async fn max_payload_test() {
		let temp_file = TempFile::new(".temp_file_server_3.ron", None);
		let server = Arc::new(HighscoreServer::new(&temp_file.path));
		let app = server.router();

		// post a new highscore
		let ron_payload = format!("(name: \"{}\", score: 5)", "x".repeat(MAX_SIZE + 1));
		let request = Request::builder()
			.method(Method::POST)
			.uri("/highscore")
			.header("content-type", "application/x-ron")
			.body(Body::from(ron_payload))
			.unwrap();
		let response = app.clone().oneshot(request).await.unwrap();
		assert_eq!(response.status(), StatusCode::BAD_REQUEST, "The post request should return as bad request");
		let body_bytes = to_bytes(response.into_body(), 2024).await.unwrap();
		let body_str = std::str::from_utf8(&body_bytes).unwrap();
		assert!(body_str.contains("Request body too large"));
	}

	#[tokio::test]
	async fn invalid_utf8_test() {
		let temp_file = TempFile::new(".temp_file_server_4.ron", None);
		let server = Arc::new(HighscoreServer::new(&temp_file.path));
		let app = server.router();

		// post a new highscore
		let ron_payload = b"(name: \"\xFF\xFF\", score: 5)".to_vec();
		let request = Request::builder()
			.method(Method::POST)
			.uri("/highscore")
			.header("content-type", "application/x-ron")
			.body(Body::from(ron_payload))
			.unwrap();
		let response = app.clone().oneshot(request).await.unwrap();
		assert_eq!(response.status(), StatusCode::BAD_REQUEST, "The post request should return as bad request");
		let body_bytes = to_bytes(response.into_body(), 2024).await.unwrap();
		let body_str = std::str::from_utf8(&body_bytes).unwrap();
		assert_eq!(body_str, "Invalid UTF-8 in request body");
	}

	#[tokio::test]
	async fn empty_name_test() {
		let temp_file = TempFile::new(".temp_file_server_5.ron", None);
		let server = Arc::new(HighscoreServer::new(&temp_file.path));
		let app = server.router();

		let ron_payload = b"(name: \"\", score: 666, level: Two)".to_vec();
		let request = Request::builder()
			.method(Method::POST)
			.uri("/highscore")
			.header("content-type", "application/x-ron")
			.body(Body::from(ron_payload))
			.unwrap();
		let response = app.clone().oneshot(request).await.unwrap();
		assert_eq!(response.status(), StatusCode::BAD_REQUEST, "The post request should return as bad request");
		let body_bytes = to_bytes(response.into_body(), 2024).await.unwrap();
		let body_str = std::str::from_utf8(&body_bytes).unwrap();
		assert_eq!(body_str, "Name cannot be empty");
	}
}
