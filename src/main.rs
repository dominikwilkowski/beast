use std::{fmt, time::Instant};

mod beasts;
mod board;
mod game;
mod help;
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

impl Tile {
	/// get the raw symbol of the tile to be displayed in the terminal
	pub fn raw_symbol(&self) -> &'static str {
		match self {
			Tile::Empty => "  ",
			Tile::Block => "░░",
			Tile::StaticBlock => "▓▓",
			Tile::Player => "◀▶",
			Tile::CommonBeast => "├┤",
			Tile::SuperBeast => "╟╢",
			Tile::Egg(_) => "○○",
			Tile::EggHatching(_) => "○○",
			Tile::HatchedBeast => "╬╬",
		}
	}
}

impl fmt::Display for Tile {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Tile::Empty => write!(f, "{}", self.raw_symbol()),
			Tile::Block => write!(f, "\x1b[32m{}\x1b[39m", self.raw_symbol()),
			Tile::StaticBlock => write!(f, "\x1b[33m{}\x1b[39m", self.raw_symbol()),
			Tile::Player => write!(f, "\x1b[36m{}\x1b[39m", self.raw_symbol()),
			Tile::CommonBeast => write!(f, "\x1b[31m{}\x1b[39m", self.raw_symbol()),
			Tile::SuperBeast => write!(f, "\x1b[31m{}\x1b[39m", self.raw_symbol()),
			Tile::Egg(_) => write!(f, "\x1b[31m{}\x1b[39m", self.raw_symbol()),
			Tile::EggHatching(_) => write!(f, "\x1b[35m{}\x1b[39m", self.raw_symbol()),
			Tile::HatchedBeast => write!(f, "\x1b[31m{}\x1b[39m", self.raw_symbol()),
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

#[cfg(test)]
mod common {
	use super::*;

	pub fn strip_ansi_border(s: &str) -> String {
		let tile_chars: Vec<char> = [
			Tile::Empty,
			Tile::Block,
			Tile::StaticBlock,
			Tile::Player,
			Tile::CommonBeast,
			Tile::SuperBeast,
			Tile::Egg(Instant::now()),
			Tile::EggHatching(Instant::now()),
			Tile::HatchedBeast,
		]
		.iter()
		.flat_map(|tile| tile.raw_symbol().chars())
		.collect();

		let mut result = String::with_capacity(s.len());
		let mut chars = s.chars().peekable();
		while let Some(c) = chars.next() {
			// check for the start of an ANSI escape sequence
			match c {
				'\x1b' => {
					if let Some(&'[') = chars.peek() {
						// consume the '['
						chars.next();
						while let Some(&ch) = chars.peek() {
							// skip over any digits or semicolons
							if ch.is_ascii_digit() || ch == ';' {
								chars.next();
							} else {
								break;
							}
						}
						// skip the final byte (usually the letter 'm')
						chars.next();
						continue;
					}
				},
				'▌' | '▐' => { /* ignore the borders */ },
				// normalize the ASCII characters we use in the game
				x if tile_chars.contains(&x) => result.push(' '),
				'●' | '←' | '→' | '⌂' | '▛' | '▀' | '▜' | '▙' | '▄' | '▟' => result.push(' '),
				// the rest is normal string stuff
				_ => result.push(c),
			}
		}
		result
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::common::strip_ansi_border;

	#[test]
	fn strip_ansi_border_16_colors_test() {
		assert_eq!(
			strip_ansi_border("\x1b[31m├┤\x1b[39m"),
			"  ",
			"strip_ansi_border should strip 16 colors ANSI escape sequences"
		);
	}

	#[test]
	fn strip_ansi_border_256_colors_test() {
		assert_eq!(
			strip_ansi_border("\x1b[38;5;82m▓▓\x1b[39m"),
			"  ",
			"strip_ansi_border should strip 256 colors ANSI escape sequences"
		);
	}

	#[test]
	fn strip_ansi_border_rgb_test() {
		assert_eq!(
			strip_ansi_border("\x1b[38;2;255;200;100m○○\x1b[39m"),
			"  ",
			"strip_ansi_border should strip rgb colors ANSI escape sequences"
		);
	}

	#[test]
	fn strip_ansi_border_tile_test() {
		let tiles = [
			Tile::Empty,
			Tile::Block,
			Tile::StaticBlock,
			Tile::Player,
			Tile::CommonBeast,
			Tile::SuperBeast,
			Tile::Egg(Instant::now()),
			Tile::EggHatching(Instant::now()),
			Tile::HatchedBeast,
		];

		for tile in &tiles {
			assert_eq!(&strip_ansi_border(&tile.to_string()), "  ", "strip_ansi_border should normalize the {:?} tile", tile);
		}
	}

	#[test]
	fn tiles_are_consistent_length_test() {
		let tiles = [
			Tile::Empty,
			Tile::Block,
			Tile::StaticBlock,
			Tile::Player,
			Tile::CommonBeast,
			Tile::SuperBeast,
			Tile::Egg(Instant::now()),
			Tile::EggHatching(Instant::now()),
			Tile::HatchedBeast,
		];

		for tile in &tiles {
			assert_eq!(tile.raw_symbol().chars().count(), 2, "tiles should be consistent length");
		}
	}
}
