//! ```shell
//!   ╔╗  ╔═╗ ╔═╗ ╔═╗ ╔╦╗
//!   ╠╩╗ ║╣  ╠═╣ ╚═╗  ║
//!   ╚═╝ ╚═╝ ╩ ╩ ╚═╝  ╩
//! ```
//!
//! > BEAST is a homage to the 1984 ASCII game "[BEAST](https://en.wikipedia.org/wiki/Beast_(video_game))"
//! > from Dan Baker, Alan Brown, Mark Hamilton and Derrick Shadel.

use dotenv::dotenv;
use std::{env, fmt, time::Instant};

mod beasts;
mod board;
mod common;
mod game;
mod help;
mod highscore;
mod pathing;
mod player;
mod stty;

/// the board width
pub const BOARD_WIDTH: usize = 50;
/// the board height
pub const BOARD_HEIGHT: usize = 30;
/// where the player starts from
pub const PLAYER_START: Coord = Coord {
	column: 0,
	row: BOARD_HEIGHT - 1,
};
/// ANSI escape sequence for bold
pub const ANSI_BOLD: &str = "\x1B[1m";
/// ANSI escape sequence to reset all styles and colors
pub const ANSI_RESET: &str = "\x1B[0m";
/// ANSI escape sequence to reset font color
pub const ANSI_RESET_FONT: &str = "\x1B[39m";
/// ANSI escape sequence to reset background color
pub const ANSI_RESET_BG: &str = "\x1B[49m";
/// left border with color ANSI escape sequence
pub const ANSI_LEFT_BORDER: &str = "\x1b[33m▌\x1b[39m";
/// right border with color ANSI escape sequence
pub const ANSI_RIGHT_BORDER: &str = "\x1b[33m▐\x1b[39m";
/// the logo
pub const LOGO: [&str; 10] = [
	"\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m",
	"\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m",
	"\x1b[33m▌\x1b[39m                               HHHH    HHHHH    HHH     HHHH   HHHHH                                \x1b[33m▐\x1b[39m",
	"\x1b[33m▌\x1b[39m                               H   H   H       H   H   H         H                                  \x1b[33m▐\x1b[39m",
	"\x1b[33m▌\x1b[39m                               H   H   H       H   H   H         H                                  \x1b[33m▐\x1b[39m",
	"\x1b[33m▌\x1b[39m                               HHHH    HHHH    HHHHH    HHH      H                                  \x1b[33m▐\x1b[39m",
	"\x1b[33m▌\x1b[39m                               H   H   H       H   H       H     H                                  \x1b[33m▐\x1b[39m",
	"\x1b[33m▌\x1b[39m                               H   H   H       H   H       H     H                                  \x1b[33m▐\x1b[39m",
	"\x1b[33m▌\x1b[39m                               HHHH    HHHHH   H   H   HHHH      H                                  \x1b[33m▐\x1b[39m",
	"\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m",
];

/// a data structure to place items on a board
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
	/// a common beast `├┤`
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
	/// moving up
	Up,
	/// moving right
	Right,
	/// moving down
	Down,
	/// moving left
	Left,
}

fn main() {
	let cli_flags = env::args().skip(1).collect::<Vec<String>>();
	if cli_flags.contains(&String::from("--version"))
		|| cli_flags.contains(&String::from("-v"))
		|| cli_flags.contains(&String::from("-V"))
	{
		println!("v{}", env!("CARGO_PKG_VERSION"));
		std::process::exit(0);
	}

	if !stty::has_stty() && std::env::var_os("CI").is_none() {
		eprintln!("\x1B[31mERROR:{ANSI_RESET_FONT} This game requires a POSIX compatible terminal with stty support.");
		std::process::exit(0);
	}

	if std::env::var_os("CI").is_none() {
		if let Ok((columns, rows)) = stty::terminal_size() {
			let min_width = game::ANSI_FRAME_SIZE + (BOARD_WIDTH * 2) + game::ANSI_FRAME_SIZE;
			let min_height = game::ANSI_HEADER_HEIGHT
				+ game::ANSI_FRAME_SIZE
				+ BOARD_HEIGHT
				+ game::ANSI_FRAME_SIZE
				+ game::ANSI_FOOTER_HEIGHT
				+ 2; // the extra space at the top and bottom
			if columns < min_width || rows < min_height {
				let width_color = if columns < min_width {
					"\x1B[31m"
				} else {
					ANSI_RESET_FONT
				};
				let height_color = if rows < min_height { "\x1B[31m" } else { ANSI_RESET_FONT };
				eprintln!(
					"\x1B[31mERROR:{ANSI_RESET_FONT} Terminal size is too small.\nThe size is {width_color}{columns}{ANSI_RESET_FONT} x {height_color}{rows}{ANSI_RESET_FONT} but needs to be at least {min_width} x {min_height}."
				);
				std::process::exit(0);
			}
		} else {
			println!("{:?}", stty::terminal_size());
			eprintln!("\x1B[31mERROR:{ANSI_RESET_FONT} Failed to detect terminal size via stty.");
			std::process::exit(0);
		}
	}

	dotenv().ok();
	let mut game = crate::game::Game::new();
	game.play();
}

#[cfg(test)]
mod test_common {
	use super::*;

	pub fn strip_ansi_border(s: &str) -> String {
		let tile_chars = [
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
		.collect::<Vec<char>>();

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
				'●' | '←' | '→' | '↓' | '↑' | '⌂' | '▛' | '▀' | '▜' | '▙' | '▄' | '▟' | '┌' | '─' | '┐' | '└' | '┘' | '│' => {
					result.push(' ')
				},
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
	use crate::test_common::strip_ansi_border;

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
