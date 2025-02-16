use crate::{board::Board, Level, Tile, BOARD_HEIGHT, BOARD_WIDTH};

pub struct Game {
	pub board: Board,
}

impl Game {
	pub fn new() -> Self {
		Self {
			board: Board::new(Level::One),
		}
	}

	pub fn play() {}

	pub fn render() {}
	fn render_header() {}
	fn render_footer() {}
}
