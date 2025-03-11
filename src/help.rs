use std::{fmt, time::Instant};

use crate::{
	Tile,
	game::{ANSI_BOARD_HEIGHT, ANSI_BOLD, ANSI_FOOTER_HEIGHT, ANSI_FRAME_HEIGHT, ANSI_RESET},
};

enum Pages {
	One,
	Two,
	Three,
}

impl fmt::Display for Pages {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Pages::One => write!(f, "● ○ ○"),
			Pages::Two => write!(f, "○ ● ○"),
			Pages::Three => write!(f, "○ ○ ●"),
		}
	}
}

const ANSI_HELP_HEIGHT: usize = 17;
const ANSI_HELP_INDEX_HEIGHT: usize = 2;

pub struct Help {
	page: Pages,
}

impl Help {
	pub fn new() -> Self {
		Self { page: Pages::One }
	}

	pub fn next_page(&mut self) {
		match self.page {
			Pages::One => self.page = Pages::Two,
			Pages::Two => self.page = Pages::Three,
			Pages::Three => self.page = Pages::One,
		}
	}

	pub fn previous_page(&mut self) {
		match self.page {
			Pages::One => self.page = Pages::Three,
			Pages::Two => self.page = Pages::One,
			Pages::Three => self.page = Pages::Two,
		}
	}

	pub fn render(&self) -> String {
		match self.page {
			Pages::One => self.page1(),
			Pages::Two => self.page2(),
			Pages::Three => self.page3(),
		}
	}

	fn page1(&self) -> String {
		let mut output = String::new();
		let top_pos = format!("\x1b[{}F", ANSI_FRAME_HEIGHT + ANSI_BOARD_HEIGHT + ANSI_FRAME_HEIGHT + ANSI_FOOTER_HEIGHT);
		let bottom_pos = format!("\x1b[{}E", ANSI_FRAME_HEIGHT + ANSI_FOOTER_HEIGHT);

		output.push_str(&top_pos);
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                HHHH    HHHHH   HHH    HHHH  HHHHH                                  \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                H   H   H      H   H  H        H                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                H   H   H      H   H  H        H                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                HHHH    HHHH   HHHHH   HHH     H                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                H   H   H      H   H      H    H                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                H   H   H      H   H      H    H                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                HHHH    HHHHH  H   H  HHHH     H                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m                                               {ANSI_BOLD}HELP{ANSI_RESET}                                                 \x1b[33m▐\x1b[39m\n"));
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m  {ANSI_BOLD}GENERAL{ANSI_RESET}                                                                                           \x1b[33m▐\x1b[39m\n"));
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m  You must survive while {ANSI_BOLD}beasts{ANSI_RESET} attack you. The only way to fight back is to squish the beasts      \x1b[33m▐\x1b[39m\n"));
		output.push_str("\x1b[33m▌\x1b[39m  between blocks. But there are different types of beasts that attack you the longer you survive.   \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m  You are {} and you move around with the arrow keys on your keyboard.                              \x1b[33m▐\x1b[39m\n", Tile::Player));
		output.push_str(&format!("\x1b[33m▌\x1b[39m  You can push {} around the board.                                                                 \x1b[33m▐\x1b[39m\n", Tile::Block));
		output.push_str(&format!("\x1b[33m▌\x1b[39m  However, {} can't be moved.                                                                       \x1b[33m▐\x1b[39m\n", Tile::StaticBlock));
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m  Your goal is to use the blocks to squish all beasts before the time runs out.                     \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m  Each level will introduce new Beasts and an ever changing environment.                            \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m  And you better hurry up because you only got a little time to survive in {ANSI_BOLD}BEAST{ANSI_RESET}.                   \x1b[33m▐\x1b[39m\n"));
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&self.render_pagination());
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m                     {ANSI_BOLD}[SPACE]{ANSI_RESET} Play  {ANSI_BOLD}[Q]{ANSI_RESET} Quit  {ANSI_BOLD}[←]{ANSI_RESET} Previous Page  {ANSI_BOLD}[→]{ANSI_RESET} Next Page                       \x1b[33m▐\x1b[39m\n"));
		output.push_str(&bottom_pos);

		output
	}

	fn page2(&self) -> String {
		let mut output = String::new();
		let top_pos = format!("\x1b[{}F", ANSI_FRAME_HEIGHT + ANSI_HELP_HEIGHT + ANSI_FRAME_HEIGHT + ANSI_FOOTER_HEIGHT);
		let bottom_pos = format!("\x1b[{}E", ANSI_FRAME_HEIGHT + ANSI_HELP_INDEX_HEIGHT + ANSI_FOOTER_HEIGHT);

		output.push_str(&top_pos);
		output.push_str(&format!("\x1b[33m▌\x1b[39m  {ANSI_BOLD}ENEMIES{ANSI_RESET}                                                                                           \x1b[33m▐\x1b[39m\n"));
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m  The {ANSI_BOLD}Common Beast{ANSI_RESET} {}                                                                               \x1b[33m▐\x1b[39m\n", Tile::CommonBeast));
		output.push_str("\x1b[33m▌\x1b[39m  It is the beast that attacks you first and in large numbers. Don't worry though, it isn't super   \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m  smart and often gets stuck. You can kill it by squishing it against any block or the board frame. \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m  The {ANSI_BOLD}Super Beast{ANSI_RESET} {}                                                                                \x1b[33m▐\x1b[39m\n", Tile::SuperBeast));
		output.push_str("\x1b[33m▌\x1b[39m  This beast is vicious and smart and will find you if you leave an opening.                        \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m  It can only be killed by squishing it against a {}.                                               \x1b[33m▐\x1b[39m\n", Tile::StaticBlock));
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m  The {ANSI_BOLD}Eggs{ANSI_RESET} {} and the {ANSI_BOLD}Hatched Beast{ANSI_RESET} {}                                                              \x1b[33m▐\x1b[39m\n", Tile::Egg(Instant::now()), Tile::HatchedBeast));
		output.push_str(&format!("\x1b[33m▌\x1b[39m  Towards the end you will encounter eggs which hatch into Hatched Beasts. These beasts can push {} \x1b[33m▐\x1b[39m\n", Tile::Block));
		output.push_str("\x1b[33m▌\x1b[39m  and will try to squish YOU with them. They can be killed like the common beasts though.           \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&self.render_pagination());
		output.push_str(&bottom_pos);

		output
	}

	// TODO: fill page
	fn page3(&self) -> String {
		let mut output = String::new();
		let top_pos = format!("\x1b[{}F", ANSI_FRAME_HEIGHT + ANSI_HELP_HEIGHT + ANSI_FRAME_HEIGHT + ANSI_FOOTER_HEIGHT);
		let bottom_pos = format!("\x1b[{}E", ANSI_FRAME_HEIGHT + ANSI_HELP_INDEX_HEIGHT + ANSI_FOOTER_HEIGHT);

		output.push_str(&top_pos);
		output.push_str(&format!("\x1b[33m▌\x1b[39m  {ANSI_BOLD}SCORING{ANSI_RESET}                                                                                           \x1b[33m▐\x1b[39m\n"));
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&self.render_pagination());
		output.push_str(&bottom_pos);

		output
	}

	fn render_pagination(&self) -> String {
		format!(
			"\x1b[33m▌\x1b[39m                                                {}                                               \x1b[33m▐\x1b[39m\n",
			self.page
		)
	}
}
