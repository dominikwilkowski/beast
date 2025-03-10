use std::{fmt, time::Instant};

mod beasts;
mod board;
mod game;
mod levels;
mod player;
mod raw_mode;

/// the board width
pub const BOARD_WIDTH: usize = 50;
/// the board height
pub const BOARD_HEIGHT: usize = 30;
/// where the player starts from
pub const PLAYER_START: Coord = Coord {
	column: 0,
	row: BOARD_HEIGHT - 1,
};

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
	/// a block `░░`
	Block,
	/// a immovable block `▓▓`
	StaticBlock,
	/// the player `◀▶`
	Player,
	/// a beast `├┤`
	CommonBeast,
	/// a super beast `╟╢`
	SuperBeast,
	/// an egg `○○`
	Egg(Instant),
	/// an egg hatching `○○` (in a different color)
	EggHatching(Instant),
	/// a hatched beast `╬╬`
	HatchedBeast,
}

impl fmt::Display for Tile {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Tile::Empty => write!(f, "  "),
			Tile::Block => write!(f, "\x1b[32m░░\x1b[39m"),
			Tile::StaticBlock => write!(f, "\x1b[33m▓▓\x1b[39m"),
			Tile::Player => write!(f, "\x1b[36m◀▶\x1b[39m"),
			Tile::CommonBeast => write!(f, "\x1b[31m├┤\x1b[39m"),
			Tile::SuperBeast => write!(f, "\x1b[31m╟╢\x1b[39m"),
			Tile::Egg(_) => write!(f, "\x1b[31m○○\x1b[39m"),
			Tile::EggHatching(_) => write!(f, "\x1b[35m○○\x1b[39m"),
			Tile::HatchedBeast => write!(f, "\x1b[31m╬╬\x1b[39m"),
		}
	}
}

/// the allowed directions an entity can move
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
	Up,
	Right,
	Down,
	Left,
}

fn main() {
	let mut game = crate::game::Game::new();
	game.play();
}
