//! this module contains the level configuration
use serde::{Deserialize, Serialize};

use std::{fmt, time::Duration};

/// game levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Level {
	One,
	Two,
	Three,
	Four,
	Five,
	Six,
	Seven,
	Eight,
	Nine,
	Ten,
}

impl Level {
	pub fn get_config(&self) -> LevelConfig {
		match self {
			Self::One => LEVEL_ONE,
			Self::Two => LEVEL_TWO,
			Self::Three => LEVEL_THREE,
			Self::Four => LEVEL_FOUR,
			Self::Five => LEVEL_FIVE,
			Self::Six => LEVEL_SIX,
			Self::Seven => LEVEL_SEVEN,
			Self::Eight => LEVEL_EIGHT,
			Self::Nine => LEVEL_NINE,
			Self::Ten => LEVEL_TEN,
		}
	}

	pub fn next(&self) -> Option<Self> {
		match self {
			Self::One => Some(Self::Two),
			Self::Two => Some(Self::Three),
			Self::Three => Some(Self::Four),
			Self::Four => Some(Self::Five),
			Self::Five => Some(Self::Six),
			Self::Six => Some(Self::Seven),
			Self::Seven => Some(Self::Eight),
			Self::Eight => Some(Self::Nine),
			Self::Nine => Some(Self::Ten),
			Self::Ten => None,
		}
	}
}

impl fmt::Display for Level {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Level::One => write!(f, "1"),
			Level::Two => write!(f, "2"),
			Level::Three => write!(f, "3"),
			Level::Four => write!(f, "4"),
			Level::Five => write!(f, "5"),
			Level::Six => write!(f, "6"),
			Level::Seven => write!(f, "7"),
			Level::Eight => write!(f, "8"),
			Level::Nine => write!(f, "9"),
			Level::Ten => write!(f, "10"),
		}
	}
}

/// level configuration
#[derive(Debug, Clone)]
pub struct LevelConfig {
	/// how many blocks are placed on the board
	pub blocks: usize,
	/// how many static blocks are placed on the board
	pub static_blocks: usize,
	/// how many common beasts are placed on the board
	pub common_beasts: usize,
	/// how many super beasts are placed on the board
	pub super_beasts: usize,
	/// how many eggs are placed on the board
	pub eggs: usize,
	/// how long it takes for an egg to hatch
	pub egg_hatching_time: Duration,
	/// how far away from each other the beasts start
	pub beast_starting_distance: usize,
	/// how long the level lasts
	pub time: Duration,
	/// how many points are awarded for completing the level
	pub completion_score: u16,
}

pub const LEVEL_ONE: LevelConfig = LevelConfig {
	blocks: 300,
	static_blocks: 10,
	common_beasts: 3,
	super_beasts: 0,
	eggs: 0,
	egg_hatching_time: Duration::from_millis(20000),
	beast_starting_distance: 16,
	time: Duration::from_secs(120),
	completion_score: 5,
};

pub const LEVEL_TWO: LevelConfig = LevelConfig {
	blocks: 250,
	static_blocks: 12,
	common_beasts: 5,
	super_beasts: 0,
	eggs: 0,
	egg_hatching_time: Duration::from_millis(20000),
	beast_starting_distance: 42,
	time: Duration::from_secs(120),
	completion_score: 7,
};

pub const LEVEL_THREE: LevelConfig = LevelConfig {
	blocks: 200,
	static_blocks: 20,
	common_beasts: 12,
	super_beasts: 0,
	eggs: 0,
	egg_hatching_time: Duration::from_millis(20000),
	beast_starting_distance: 27,
	time: Duration::from_secs(240),
	completion_score: 7,
};

pub const LEVEL_FOUR: LevelConfig = LevelConfig {
	blocks: 180,
	static_blocks: 30,
	common_beasts: 10,
	super_beasts: 1,
	eggs: 0,
	egg_hatching_time: Duration::from_millis(20000),
	beast_starting_distance: 27,
	time: Duration::from_secs(240),
	completion_score: 10,
};

pub const LEVEL_FIVE: LevelConfig = LevelConfig {
	blocks: 170,
	static_blocks: 30,
	common_beasts: 10,
	super_beasts: 3,
	eggs: 0,
	egg_hatching_time: Duration::from_millis(20000),
	beast_starting_distance: 27,
	time: Duration::from_secs(240),
	completion_score: 12,
};

pub const LEVEL_SIX: LevelConfig = LevelConfig {
	blocks: 160,
	static_blocks: 30,
	common_beasts: 10,
	super_beasts: 7,
	eggs: 0,
	egg_hatching_time: Duration::from_millis(20000),
	beast_starting_distance: 27,
	time: Duration::from_secs(300),
	completion_score: 15,
};

pub const LEVEL_SEVEN: LevelConfig = LevelConfig {
	blocks: 160,
	static_blocks: 50,
	common_beasts: 5,
	super_beasts: 1,
	eggs: 1,
	egg_hatching_time: Duration::from_millis(20000),
	beast_starting_distance: 27,
	time: Duration::from_secs(300),
	completion_score: 20,
};

pub const LEVEL_EIGHT: LevelConfig = LevelConfig {
	blocks: 160,
	static_blocks: 100,
	common_beasts: 10,
	super_beasts: 5,
	eggs: 3,
	egg_hatching_time: Duration::from_millis(20000),
	beast_starting_distance: 27,
	time: Duration::from_secs(330),
	completion_score: 25,
};

pub const LEVEL_NINE: LevelConfig = LevelConfig {
	blocks: 150,
	static_blocks: 150,
	common_beasts: 10,
	super_beasts: 5,
	eggs: 5,
	egg_hatching_time: Duration::from_millis(17000),
	beast_starting_distance: 27,
	time: Duration::from_secs(330),
	completion_score: 30,
};

pub const LEVEL_TEN: LevelConfig = LevelConfig {
	blocks: 180,
	static_blocks: 150,
	common_beasts: 10,
	super_beasts: 10,
	eggs: 8,
	egg_hatching_time: Duration::from_millis(10000),
	beast_starting_distance: 27,
	time: Duration::from_secs(360),
	completion_score: 100,
};

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn level_display_test() {
		assert_eq!(Level::One.to_string(), "1", "This level should render to 1");
		assert_eq!(Level::Two.to_string(), "2", "This level should render to 2");
		assert_eq!(Level::Three.to_string(), "3", "This level should render to 3");
		assert_eq!(Level::Four.to_string(), "4", "This level should render to 4");
		assert_eq!(Level::Five.to_string(), "5", "This level should render to 5");
		assert_eq!(Level::Six.to_string(), "6", "This level should render to 6");
		assert_eq!(Level::Seven.to_string(), "7", "This level should render to 7");
		assert_eq!(Level::Eight.to_string(), "8", "This level should render to 8");
		assert_eq!(Level::Nine.to_string(), "9", "This level should render to 9");
		assert_eq!(Level::Ten.to_string(), "10", "This level should render to 10");
	}

	#[test]
	fn level_next_test() {
		assert_eq!(Level::One.next(), Some(Level::Two), "This level should progress to 2");
		assert_eq!(Level::Two.next(), Some(Level::Three), "This level should progress to 3");
		assert_eq!(Level::Three.next(), Some(Level::Four), "This level should progress to 4");
		assert_eq!(Level::Four.next(), Some(Level::Five), "This level should progress to 5");
		assert_eq!(Level::Five.next(), Some(Level::Six), "This level should progress to 6");
		assert_eq!(Level::Six.next(), Some(Level::Seven), "This level should progress to 7");
		assert_eq!(Level::Seven.next(), Some(Level::Eight), "This level should progress to 8");
		assert_eq!(Level::Eight.next(), Some(Level::Nine), "This level should progress to 9");
		assert_eq!(Level::Nine.next(), Some(Level::Ten), "This level should progress to 10");
		assert_eq!(Level::Ten.next(), None, "This level should be the last level");
	}
}
