mod beasts;
mod board;
mod game;

/// the board width
pub const BOARD_WIDTH: usize = 100;
/// the board height
pub const BOARD_HEIGHT: usize = 40;
/// where the player starts from
pub const PLAYER_START: Coord = Coord { x: 1, y: 32 };

/// a data structure to place items on a board
pub struct Coord {
	x: usize,
	y: usize,
}

/// the items that can be found on the baord
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
	/// empty space
	Empty,
	/// a block ▓▓
	Block,
	/// a immovable block ▓▓
	ImmovableBlock,
	/// the player ◄►
	Player,
	/// a beast ├┤
	Beast,
	/// a super beast ╟╢
	SuperBeast,
	/// a super beast egg ○○ ⚬⚬ ◦◦ ╟╢
	Egg,
	/// a hatched beast ╬╬
	HatchedBeast,
}

/// game levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
	One,
	Two,
	Three,
}

fn main() {
	println!("Hello, world!");
}
