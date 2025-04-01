use ron::de::from_str;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct Highscore {
	#[serde(with = "time::serde::rfc3339")]
	pub timestamp: OffsetDateTime,
	pub name: String,
	pub score: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Highscores {
	pub scores: Vec<Highscore>,
}

impl Highscores {
	pub fn ron_from_str(s: &str) -> Result<Self, ron::Error> {
		Ok(from_str::<Self>(s)?)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_highscores() {
		let ron_str = r#"
			(
				scores: [
					(
						timestamp: "2023-04-01T12:34:56Z",
						name: "Dom",
						score: 42,
					),
					(
						timestamp: "2023-04-02T10:00:00Z",
						name: "Alan",
						score: 666,
					),
				],
			)"#;

		let highscores = Highscores::ron_from_str(ron_str).expect("Failed to parse RON");
		assert_eq!(highscores.scores.len(), 2, "The parsed struct should have two items in scores");
		assert_eq!(highscores.scores[0].name, "Dom", "The first highscore should have the name 'Dom'");
		assert_eq!(highscores.scores[0].score, 42, "The first highscore should have the score 42");
		assert_eq!(highscores.scores[1].name, "Alan", "The second highscore should have the name 'Alan'");
		assert_eq!(highscores.scores[1].score, 666, "The second highscore should have the score 666");
	}
}
