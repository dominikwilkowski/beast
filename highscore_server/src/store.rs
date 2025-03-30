use ron::{de::from_str, ser::to_string};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::Arc};
use time::OffsetDateTime;
use tokio::sync::Mutex;

use crate::errors::HighscoreError;

const MAX_SCORES: usize = 100;
const MAX_NAME_LENGTH: usize = 50;

#[derive(Serialize, Deserialize)]
pub struct Highscore {
	#[serde(with = "time::serde::rfc3339")]
	timestamp: OffsetDateTime,
	pub name: String,
	pub score: u16,
}

#[derive(Serialize, Deserialize)]
pub struct Highscores {
	pub scores: Vec<Highscore>,
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
			Err(error) => panic!("Reading highscore db at {db_path:?} failed: {error}"),
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

			scores.scores.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.timestamp.cmp(&b.timestamp)));
			if scores.scores.len() > MAX_SCORES {
				scores.scores.truncate(MAX_SCORES);
			}

			let ron_str = to_string(&*scores).map_err(HighscoreError::SerializationError)?;

			let temp_path = self.db_path.with_extension("tmp");
			tokio::fs::write(&temp_path, &ron_str).await.map_err(HighscoreError::FileWriteError)?;
			tokio::fs::rename(&temp_path, &self.db_path).await.map_err(HighscoreError::FileRenameError)?;
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::common::TempFile;

	#[tokio::test]
	async fn add_get_score_test() {
		let temp_file = TempFile::new(".temp_file_store_1.ron", None);
		let store = HighscoreStore::new(&temp_file.path);

		store.add_score(String::from(r#"(name:"TestPlayer",score:555)"#)).await.unwrap();

		let scores_return = store.get_scores().await.unwrap();
		let scores = from_str::<Highscores>(&scores_return).unwrap();

		assert_eq!(scores.scores.len(), 1, "We should have one score");
		assert_eq!(scores.scores[0].name, "TestPlayer", "The name of the score should be what we entered");
		assert_eq!(scores.scores[0].score, 555, "The score should be what we entered");
	}

	#[tokio::test]
	async fn sort_and_truncate_test() {
		let temp_file = TempFile::new(".temp_file_store_2.ron", None);
		let store = HighscoreStore::new(&temp_file.path);

		store.add_score(String::from(r#"(name:"Player1",score:100)"#)).await.unwrap();
		store.add_score(String::from(r#"(name:"Player2",score:300)"#)).await.unwrap();
		store.add_score(String::from(r#"(name:"Player3",score:200)"#)).await.unwrap();
		store.add_score(String::from(r#"(name:"Player4",score:200)"#)).await.unwrap();

		let scores_return = store.get_scores().await.unwrap();
		let scores = from_str::<Highscores>(&scores_return).unwrap();

		assert_eq!(scores.scores.len(), 4, "We should have as many scores as we entered");
		assert_eq!(scores.scores[0].name, "Player2", "The name of the highest score should be Player2");
		assert_eq!(scores.scores[0].score, 300, "The score of the highest score should be 300");
		assert_eq!(scores.scores[1].name, "Player3", "The name of the second highest score should be Player3");
		assert_eq!(scores.scores[1].score, 200, "The score of the second highest score should be 200");
		assert_eq!(scores.scores[2].name, "Player4", "The name of the third highest score should be Player4");
		assert_eq!(scores.scores[2].score, 200, "The score of the third highest score should be 200");
		assert_eq!(scores.scores[3].name, "Player1", "The name of the lowest score should be Player1");
		assert_eq!(scores.scores[3].score, 100, "The score of the lowest score should be 100");
	}

	#[tokio::test]
	async fn name_validation_test() {
		let temp_file = TempFile::new(".temp_file_store_3.ron", None);
		let store = HighscoreStore::new(&temp_file.path);

		let result = store.add_score(String::from(r#"(name:"",score:100)"#)).await;
		assert!(
			matches!(result, Err(HighscoreError::EmptyName)),
			"The store should error with 'Name cannot be empty' for an empty name"
		);

		let result = store.add_score(String::from(r#"(name:"   ",score:100)"#)).await;
		assert!(
			matches!(result, Err(HighscoreError::EmptyName)),
			"The store should error with 'Name cannot be empty' for a name with only spaces"
		);
	}

	#[tokio::test]
	async fn name_truncation_test() {
		let temp_file = TempFile::new(".temp_file_store_4.ron", None);
		let store = HighscoreStore::new(&temp_file.path);

		let long_name: String = "X".repeat(MAX_NAME_LENGTH + 10);
		let score_data = format!(r#"(name:"{long_name}",score:1)"#);

		store.add_score(score_data).await.unwrap();

		let scores_return = store.get_scores().await.unwrap();
		let scores = from_str::<Highscores>(&scores_return).unwrap();

		assert_eq!(scores.scores.len(), 1, "The store should have stored our score");
		assert_eq!(scores.scores[0].name.len(), MAX_NAME_LENGTH, "The name should be truncated to the maximum length");
		assert_eq!(scores.scores[0].score, 1, "The score should be stored correctly");
	}

	#[tokio::test]
	async fn highscore_truncation_test() {
		let fixed_scores: String =
			"(timestamp:\"2025-03-28T21:03:01.578945Z\",name:\"Old Player\",score:50),".repeat(MAX_SCORES);
		let temp_file = TempFile::new(".temp_file_store_5.ron", Some(format!("(scores:[{fixed_scores}])")));
		let store = HighscoreStore::new(&temp_file.path);

		store.add_score(String::from(r#"(name:"Dom 1",score:100)"#)).await.unwrap();
		store.add_score(String::from(r#"(name:"Dom 2",score:49)"#)).await.unwrap();
		store.add_score(String::from(r#"(name:"Dom 3",score:102)"#)).await.unwrap();

		let scores_return = store.get_scores().await.unwrap();
		let scores = from_str::<Highscores>(&scores_return).unwrap();

		assert_eq!(
			scores.scores.len(),
			MAX_SCORES,
			"The store should still have the maximum number of scores stored and not added more"
		);
		assert_eq!(scores.scores[0].name, "Dom 3", "The top score should be 'Dom 3'");
		assert_eq!(scores.scores[0].score, 102, "The top score should be 102");
		assert_eq!(scores.scores[1].name, "Dom 1", "The second score should be 'Dom 1'");
		assert_eq!(scores.scores[1].score, 100, "The second score should be 100");
		assert!(
			!scores.scores.iter().any(|entry| entry.name == "Dom 2" && entry.score == 49),
			"The entry 'Dom 2' should not exist since it is less score than in the existing scores store"
		);
	}

	#[tokio::test]
	async fn file_persistence_test() {
		let temp_file = TempFile::new(".temp_file_store_6.ron", None);

		{
			let store = HighscoreStore::new(&temp_file.path);
			store.add_score(String::from(r#"(name:"Dom",score:666)"#)).await.unwrap();
		}

		{
			let store = HighscoreStore::new(&temp_file.path);
			let scores_str = store.get_scores().await.unwrap();
			let scores: Highscores = from_str(&scores_str).unwrap();

			assert_eq!(scores.scores.len(), 1, "The store should have saved our score");
			assert_eq!(scores.scores[0].name, "Dom", "The name of the top score should be 'Dom'");
			assert_eq!(scores.scores[0].score, 666, "The score of the top score should be 666");
		}
	}
}
