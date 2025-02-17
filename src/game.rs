use crate::{board::Board, Level};

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
