//! this module contains code shared between beast and beast_highscore_server

use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, UtcOffset, format_description};

use crate::levels::Level;

pub mod levels;

/// the max length of names entered into the highscore
pub const MAX_NAME_LENGTH: usize = 50;
/// the max amount of scores we store
pub const MAX_SCORES: usize = 100;

/// the higscore type with all data that is being stored
#[derive(Debug, Serialize, Deserialize)]
pub struct Highscore {
	/// when the score was submitted
	#[serde(with = "time::serde::rfc3339")]
	pub timestamp: OffsetDateTime,
	/// the name of the submitter
	pub name: String,
	/// the score reached
	pub score: u16,
	/// the level reached
	pub level: Level,
}

impl Highscore {
	/// formatting our timestamp consistently
	pub fn format_timestamp(&self) -> String {
		let local_offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
		let local_time = self.timestamp.to_offset(local_offset);
		let format = format_description::parse("[year]-[month]-[day] [hour repr:12]:[minute] [period]")
			.expect("Invalid format description");

		local_time.format(&format).expect("Failed to format timestamp")
	}

	/// create a new instance of highscore
	pub fn new(name: &str, score: u16, level: Level) -> Self {
		Self {
			timestamp: OffsetDateTime::now_utc(),
			name: name.to_string(),
			score,
			level,
		}
	}
}

/// the score type
#[derive(Debug, Serialize, Deserialize)]
pub struct Score {
	/// the name of the submitter
	pub name: String,
	/// the score reached
	pub score: u16,
	/// the level reached
	pub level: Level,
}

/// the type for scores we send to the client
#[derive(Debug, Serialize, Deserialize)]
pub struct Highscores {
	/// all highscore data we have stored in the server
	pub scores: Vec<Highscore>,
}

impl Highscores {
	/// convert a str into ron
	pub fn ron_from_str(s: &str) -> Result<Self, ron::Error> {
		Ok(ron::from_str::<Self>(s)?)
	}

	/// convert ron into a String
	pub fn ron_to_str(data: &Score) -> Result<String, ron::Error> {
		ron::to_string(data)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn ron_from_str_test() {
		let ron_str = r#"
			(
				scores: [
					(
						timestamp: "2023-04-01T12:34:56Z",
						name: "Dom",
						score: 42,
						level: One,
					),
					(
						timestamp: "2023-04-02T10:00:00Z",
						name: "Alan",
						score: 666,
						level: Eight,
					),
				],
			)"#;

		let highscores = Highscores::ron_from_str(ron_str).expect("Failed to parse RON");
		assert_eq!(highscores.scores.len(), 2, "The parsed struct should have two items in scores");
		assert_eq!(highscores.scores[0].name, "Dom", "The first highscore should have the name 'Dom'");
		assert_eq!(highscores.scores[0].score, 42, "The first highscore should have the score 42");
		assert_eq!(highscores.scores[0].level, Level::One, "The first highscore should have the level one");
		assert_eq!(highscores.scores[1].name, "Alan", "The second highscore should have the name 'Alan'");
		assert_eq!(highscores.scores[1].score, 666, "The second highscore should have the score 666");
		assert_eq!(highscores.scores[1].level, Level::Eight, "The second highscore should have the level eight");
	}

	#[test]
	fn ron_to_str_test() {
		assert_eq!(
			Highscores::ron_to_str(&Score {
				name: String::from("Dom"),
				score: 666,
				level: Level::One,
			}),
			Ok(String::from("(name:\"Dom\",score:666,level:One)")),
			"The ron string should have include the first name"
		);
	}
}
