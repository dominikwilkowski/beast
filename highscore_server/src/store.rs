use ron::{de::from_str, ser::to_string};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::Arc};
use time::OffsetDateTime;
use tokio::sync::Mutex;

use crate::errors::HighscoreError;

const MAX_SCORES: usize = 100;
const MAX_NAME_LENGTH: usize = 255;

#[derive(Serialize, Deserialize)]
pub struct Highscore {
	#[serde(with = "time::serde::rfc3339")]
	timestamp: OffsetDateTime,
	name: String,
	score: u16,
}

#[derive(Serialize, Deserialize)]
struct Highscores {
	scores: Vec<Highscore>,
}

#[derive(Deserialize)]
pub struct ClientHighscoreData {
	name: String,
	score: u16,
}

pub struct HighscoreStore {
	inner: Arc<Mutex<Highscores>>,
	db_path: PathBuf,
}

impl HighscoreStore {
	pub fn new(db_path: impl Into<PathBuf>) -> Self {
		let db_path = db_path.into();
		let highscores = match fs::read_to_string(&db_path) {
			Ok(content) => match from_str::<Highscores>(&content) {
				Ok(scores) => scores,
				Err(error) => {
					panic!("Failed to parse highscores file: {error}");
				},
			},
			Err(error) => panic!("File read error: {error}"),
		};

		Self {
			inner: Arc::new(Mutex::new(highscores)),
			db_path,
		}
	}

	pub async fn get_scores(&self) -> Result<String, HighscoreError> {
		let scores = self.inner.lock().await;
		to_string(&*scores).map_err(HighscoreError::SerializationError)
	}

	pub async fn add_score(&self, body: String) -> Result<(), HighscoreError> {
		let data: ClientHighscoreData = match from_str(&body) {
			Ok(data) => data,
			Err(error) => return Err(HighscoreError::RonParseError(error.into())),
		};

		if data.name.trim().is_empty() {
			return Err(HighscoreError::EmptyName);
		}

		let new_entry = Highscore {
			timestamp: OffsetDateTime::now_utc(),
			name: data.name.chars().take(MAX_NAME_LENGTH).collect(),
			score: data.score,
		};

		{
			let mut scores = self.inner.lock().await;
			scores.scores.push(new_entry);

			scores.scores.sort_by(|a, b| b.score.cmp(&a.score));
			if scores.scores.len() > MAX_SCORES {
				scores.scores.truncate(MAX_SCORES);
			}

			let ron_str = to_string(&*scores).map_err(HighscoreError::SerializationError)?;

			let temp_path = self.db_path.with_extension("tmp");
			fs::write(&temp_path, &ron_str).map_err(HighscoreError::FileWriteError)?;
			fs::rename(&temp_path, &self.db_path).map_err(HighscoreError::FileRenameError)?;
		}
		Ok(())
	}
}
