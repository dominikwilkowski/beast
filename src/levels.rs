//! this module contains the level configuration

use std::{fmt, time::Duration};

/// game levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
			Level::One => LEVEL_ONE,
			Level::Two => LEVEL_TWO,
			Level::Three => LEVEL_THREE,
			Level::Four => LEVEL_FOUR,
			Level::Five => LEVEL_FIVE,
			Level::Six => LEVEL_SIX,
			Level::Seven => LEVEL_SEVEN,
			Level::Eight => LEVEL_EIGHT,
			Level::Nine => LEVEL_NINE,
			Level::Ten => LEVEL_TEN,
		}
	}

	pub fn next(&self) -> Option<Self> {
		match self {
			Level::One => Some(Level::Two),
			Level::Two => Some(Level::Three),
			Level::Three => Some(Level::Four),
			Level::Four => Some(Level::Five),
			Level::Five => Some(Level::Six),
			Level::Six => Some(Level::Seven),
			Level::Seven => Some(Level::Eight),
			Level::Eight => Some(Level::Nine),
			Level::Nine => Some(Level::Ten),
			Level::Ten => None,
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
	super_beasts: 5,
	eggs: 0,
	egg_hatching_time: Duration::from_millis(20000),
	beast_starting_distance: 27,
	time: Duration::from_secs(300),
	completion_score: 15,
};

pub const LEVEL_SEVEN: LevelConfig = LevelConfig {
	blocks: 160,
	static_blocks: 50,
	common_beasts: 10,
	super_beasts: 7,
	eggs: 0,
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
	eggs: 6,
	egg_hatching_time: Duration::from_millis(10000),
	beast_starting_distance: 27,
	time: Duration::from_secs(360),
	completion_score: 100,
};
