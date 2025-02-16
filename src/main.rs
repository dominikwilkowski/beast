use std::fmt;

mod beasts;
mod board;
mod game;

/// the board width
pub const BOARD_WIDTH: usize = 50;
/// the board height
pub const BOARD_HEIGHT: usize = 30;
/// where the player starts from
pub const PLAYER_START: Coord = Coord { column: 1, row: 32 };

/// a data structure to place items on a board
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coord {
	column: usize,
	row: usize,
}

/// the items that can be found on the baord
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
	/// empty space
	Empty,
	/// a block ░░
	Block,
	/// a immovable block ▓▓
	ImmovableBlock,
	/// the player ◄►
	Player,
	/// a beast ├┤
	Beast,
	/// a super beast ╟╢
	SuperBeast,
	/// a super beast egg ○○
	Egg,
	/// a super beast egg ○○ hatching
	EggHatching,
	/// a hatched beast ╬╬
	HatchedBeast,
}

impl fmt::Display for Tile {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Tile::Empty => write!(f, "  "),
			Tile::Block => write!(f, "\x1b[32m░░\x1b[39m"),
			Tile::ImmovableBlock => write!(f, "\x1b[33m▓▓\x1b[39m"),
			Tile::Player => write!(f, "\x1b[36m◄►\x1b[39m"),
			Tile::Beast => write!(f, "\x1b[31m├┤\x1b[39m"),
			Tile::SuperBeast => write!(f, "\x1b[31m╟╢\x1b[39m"),
			Tile::Egg => write!(f, "\x1b[35m○○\x1b[39m"),
			Tile::EggHatching => write!(f, "\x1b[31m○○\x1b[39m"),
			Tile::HatchedBeast => write!(f, "\x1b[31m╬╬\x1b[39m"),
		}
	}
}

/// game levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
	One,
	Two,
	Three,
}

/// level configuration
pub struct LevelConfig {
	pub level: Level,
	pub blocks: usize,
	pub immovable_blocks: usize,
	pub beasts: usize,
	pub super_beasts: usize,
	pub eggs: usize,
	pub beast_starting_distance: usize,
}

pub const LEVEL_ONE: LevelConfig = LevelConfig {
	level: Level::One,
	blocks: 300,
	immovable_blocks: 30,
	beasts: 3,
	super_beasts: 0,
	eggs: 0,
	beast_starting_distance: 16,
};

pub const LEVEL_TWO: LevelConfig = LevelConfig {
	level: Level::Two,
	blocks: 200,
	immovable_blocks: 50,
	beasts: 5,
	super_beasts: 0,
	eggs: 3,
	beast_starting_distance: 42,
};

pub const LEVEL_THREE: LevelConfig = LevelConfig {
	level: Level::Three,
	blocks: 180,
	immovable_blocks: 150,
	beasts: 12,
	super_beasts: 3,
	eggs: 5,
	beast_starting_distance: 27,
};

fn main() {
	let game = crate::game::Game::new();
	println!("{}", game.board.render_full());
}
