//! this module contains error handling for the server in a centralized spot

use axum::{
	http::{HeaderMap, HeaderValue, StatusCode},
	response::IntoResponse,
};
use std::{fmt, io};

#[derive(Debug)]
pub enum HighscoreError {
	// input validation errors
	EmptyName,
	InvalidUtf8,
	RequestTooLarge(usize),

	// parsing errors
	RonParseError(ron::Error),

	// storage errors
	SerializationError(ron::Error),
	FileWriteError(io::Error),
	FileRenameError(io::Error),
}

impl fmt::Display for HighscoreError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::EmptyName => write!(f, "Name cannot be empty"),
			Self::InvalidUtf8 => write!(f, "Invalid UTF-8 in request body"),
			Self::RequestTooLarge(max) => write!(f, "Request body too large (maximum {} KB)", max / 1024),
			Self::RonParseError(error) => write!(f, "RON parsing error: {error}"),
			Self::SerializationError(error) => write!(f, "Serialization error: {error}"),
			Self::FileWriteError(error) => write!(f, "File write error: {error}"),
			Self::FileRenameError(error) => write!(f, "File rename error: {error}"),
		}
	}
}

impl IntoResponse for HighscoreError {
	fn into_response(self) -> axum::response::Response {
		let status = match &self {
			Self::EmptyName | Self::InvalidUtf8 | Self::RequestTooLarge(_) | Self::RonParseError(_) => {
				StatusCode::BAD_REQUEST
			},
			_ => StatusCode::INTERNAL_SERVER_ERROR,
		};

		let mut headers = HeaderMap::new();
		headers.insert("Content-Type", HeaderValue::from_static("text/plain"));

		(status, headers, self.to_string()).into_response()
	}
}
