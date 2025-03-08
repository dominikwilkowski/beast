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
pub struct LevelConfig {
	pub blocks: usize,
	pub static_blocks: usize,
	pub common_beasts: usize,
	pub super_beasts: usize,
	pub eggs: usize,
	pub egg_hatching_time: Duration,
	pub beast_starting_distance: usize,
	pub time: Duration,
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
};

pub const LEVEL_THREE: LevelConfig = LevelConfig {
	blocks: 200,
	static_blocks: 20,
	common_beasts: 12,
	super_beasts: 0,
	eggs: 0,
	egg_hatching_time: Duration::from_millis(20000),
	beast_starting_distance: 27,
	time: Duration::from_secs(120),
};

pub const LEVEL_FOUR: LevelConfig = LevelConfig {
	blocks: 180,
	static_blocks: 30,
	common_beasts: 10,
	super_beasts: 1,
	eggs: 0,
	egg_hatching_time: Duration::from_millis(20000),
	beast_starting_distance: 27,
	time: Duration::from_secs(180),
};

pub const LEVEL_FIVE: LevelConfig = LevelConfig {
	blocks: 170,
	static_blocks: 30,
	common_beasts: 10,
	super_beasts: 3,
	eggs: 0,
	egg_hatching_time: Duration::from_millis(20000),
	beast_starting_distance: 27,
	time: Duration::from_secs(180),
};

pub const LEVEL_SIX: LevelConfig = LevelConfig {
	blocks: 160,
	static_blocks: 30,
	common_beasts: 10,
	super_beasts: 5,
	eggs: 0,
	egg_hatching_time: Duration::from_millis(20000),
	beast_starting_distance: 27,
	time: Duration::from_secs(180),
};

pub const LEVEL_SEVEN: LevelConfig = LevelConfig {
	blocks: 160,
	static_blocks: 50,
	common_beasts: 10,
	super_beasts: 7,
	eggs: 0,
	egg_hatching_time: Duration::from_millis(20000),
	beast_starting_distance: 27,
	time: Duration::from_secs(180),
};

pub const LEVEL_EIGHT: LevelConfig = LevelConfig {
	blocks: 160,
	static_blocks: 100,
	common_beasts: 10,
	super_beasts: 5,
	eggs: 3,
	egg_hatching_time: Duration::from_millis(20000),
	beast_starting_distance: 27,
	time: Duration::from_secs(240),
};

pub const LEVEL_NINE: LevelConfig = LevelConfig {
	blocks: 150,
	static_blocks: 150,
	common_beasts: 10,
	super_beasts: 5,
	eggs: 5,
	egg_hatching_time: Duration::from_millis(17000),
	beast_starting_distance: 27,
	time: Duration::from_secs(240),
};

pub const LEVEL_TEN: LevelConfig = LevelConfig {
	blocks: 180,
	static_blocks: 150,
	common_beasts: 10,
	super_beasts: 10,
	eggs: 6,
	egg_hatching_time: Duration::from_millis(10000),
	beast_starting_distance: 27,
	time: Duration::from_secs(240),
};
