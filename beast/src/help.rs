//! this module allows to display paginated help in the CLI
use std::{fmt, time::Instant};

use crate::{
	LOGO, Tile,
	beasts::{Beast, CommonBeast, Egg, HatchedBeast, SuperBeast},
	game::{ANSI_BOARD_HEIGHT, ANSI_BOLD, ANSI_FOOTER_HEIGHT, ANSI_FRAME_SIZE, ANSI_RESET},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Page {
	One,
	Two,
	Three,
}

impl fmt::Display for Page {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Page::One => write!(f, "● ○ ○"),
			Page::Two => write!(f, "○ ● ○"),
			Page::Three => write!(f, "○ ○ ●"),
		}
	}
}

const ANSI_HELP_HEIGHT: usize = 17;
const ANSI_HELP_INDEX_HEIGHT: usize = 2;

pub struct Help {
	page: Page,
}

impl Help {
	pub fn new() -> Self {
		Self { page: Page::One }
	}

	pub fn next_page(&mut self) {
		match self.page {
			Page::One => self.page = Page::Two,
			Page::Two => self.page = Page::Three,
			Page::Three => self.page = Page::One,
		}
	}

	pub fn previous_page(&mut self) {
		match self.page {
			Page::One => self.page = Page::Three,
			Page::Two => self.page = Page::One,
			Page::Three => self.page = Page::Two,
		}
	}

	pub fn render(&self) -> String {
		match self.page {
			Page::One => self.general_page(),
			Page::Two => self.beast_page(),
			Page::Three => self.scoring_page(),
		}
	}

	fn general_page(&self) -> String {
		let mut output = String::new();
		let top_pos = format!("\x1b[{}F", ANSI_FRAME_SIZE + ANSI_BOARD_HEIGHT + ANSI_FRAME_SIZE + ANSI_FOOTER_HEIGHT);
		let bottom_pos = format!("\x1b[{}E", ANSI_FRAME_SIZE + ANSI_FOOTER_HEIGHT);

		output.push_str(&top_pos);
		output.push_str(&LOGO.join("\n"));
		output.push('\n');
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
		output.push_str(&format!("\x1b[33m▌\x1b[39m             {ANSI_BOLD}[SPACE]{ANSI_RESET} Play  {ANSI_BOLD}[Q]{ANSI_RESET} Quit  {ANSI_BOLD}[S]{ANSI_RESET} Highscores  {ANSI_BOLD}[←]{ANSI_RESET} Previous Page  {ANSI_BOLD}[→]{ANSI_RESET} Next Page               \x1b[33m▐\x1b[39m\n"));
		output.push_str(&bottom_pos);

		output
	}

	fn beast_page(&self) -> String {
		let mut output = String::new();
		let top_pos = format!("\x1b[{}F", ANSI_FRAME_SIZE + ANSI_HELP_HEIGHT + ANSI_FRAME_SIZE + ANSI_FOOTER_HEIGHT);
		let bottom_pos = format!("\x1b[{}E", ANSI_FRAME_SIZE + ANSI_HELP_INDEX_HEIGHT + ANSI_FOOTER_HEIGHT);

		output.push_str(&top_pos);
		output.push_str(&format!("\x1b[33m▌\x1b[39m  {ANSI_BOLD}ENEMIES{ANSI_RESET}                                                                                           \x1b[33m▐\x1b[39m\n"));
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m  The {ANSI_BOLD}Common Beast{ANSI_RESET} {}                                                                               \x1b[33m▐\x1b[39m\n", Tile::CommonBeast));
		output.push_str("\x1b[33m▌\x1b[39m  It's the beast that attacks you first and in large numbers. Don't worry though, it isn't super    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m  smart and often gets stuck. You can kill it by squishing it against any block or the board frame. \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m  The {ANSI_BOLD}Super Beast{ANSI_RESET} {}                                                                                \x1b[33m▐\x1b[39m\n", Tile::SuperBeast));
		output.push_str("\x1b[33m▌\x1b[39m  This beast is vicious and smart and will find you if you leave an opening.                        \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m  It can only be killed by squishing it against a {}.                                               \x1b[33m▐\x1b[39m\n", Tile::StaticBlock));
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m  The {ANSI_BOLD}Egg{ANSI_RESET} {} and the {ANSI_BOLD}Hatched Beast{ANSI_RESET} {}                                                               \x1b[33m▐\x1b[39m\n", Tile::Egg(Instant::now()), Tile::HatchedBeast));
		output.push_str(&format!("\x1b[33m▌\x1b[39m  Towards the end you will encounter eggs which hatch into Hatched Beasts. These beasts can push {} \x1b[33m▐\x1b[39m\n", Tile::Block));
		output.push_str("\x1b[33m▌\x1b[39m  and will try to squish YOU with them. They can be killed like the common beasts though.           \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&self.render_pagination());
		output.push_str(&bottom_pos);

		output
	}

	fn scoring_page(&self) -> String {
		let mut output = String::new();
		let top_pos = format!("\x1b[{}F", ANSI_FRAME_SIZE + ANSI_HELP_HEIGHT + ANSI_FRAME_SIZE + ANSI_FOOTER_HEIGHT);
		let bottom_pos = format!("\x1b[{}E", ANSI_FRAME_SIZE + ANSI_HELP_INDEX_HEIGHT + ANSI_FOOTER_HEIGHT);

		output.push_str(&top_pos);
		output.push_str(&format!("\x1b[33m▌\x1b[39m  {ANSI_BOLD}SCORING{ANSI_RESET}                                                                                           \x1b[33m▐\x1b[39m\n"));
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m  You add scores by squishing beasts, completing levels and having time left over by the end of     \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m  level. Additionally each second you have left over after you finished a level                     \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m  will award you 0.1 score.                                                                         \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m  Beast  | Score for squishing                                                                      \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m  ----------------------------                                                                      \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m  {}     | {}                                                                                        \x1b[33m▐\x1b[39m\n", Tile::CommonBeast, CommonBeast::get_score()));
		output.push_str(&format!("\x1b[33m▌\x1b[39m  {}     | {}                                                                                        \x1b[33m▐\x1b[39m\n", Tile::SuperBeast, SuperBeast::get_score()));
		output.push_str(&format!("\x1b[33m▌\x1b[39m  {}     | {}                                                                                        \x1b[33m▐\x1b[39m\n", Tile::Egg(Instant::now()), Egg::get_score()));
		output.push_str(&format!("\x1b[33m▌\x1b[39m  {}     | {}                                                                                        \x1b[33m▐\x1b[39m\n", Tile::HatchedBeast, HatchedBeast::get_score()));
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

#[cfg(test)]
mod test {
	use super::*;
	use crate::{BOARD_WIDTH, common::strip_ansi_border};

	#[test]
	fn next_page_test() {
		let mut help = Help::new();
		assert_eq!(help.page, Page::One, "The first page should be page one");
		help.next_page();
		assert_eq!(help.page, Page::Two, "The page after calling next_page 1 time should be page two");
		help.next_page();
		assert_eq!(help.page, Page::Three, "The page after calling next_page 2 time should be page three");
		help.next_page();
		assert_eq!(help.page, Page::One, "The page after calling next_page 3 time should be page one");
	}

	#[test]
	fn previous_page_test() {
		let mut help = Help::new();
		assert_eq!(help.page, Page::One, "The first page should be page one");
		help.previous_page();
		assert_eq!(help.page, Page::Three, "The page after calling previous_page 1 time should be page three");
		help.previous_page();
		assert_eq!(help.page, Page::Two, "The page after calling previous_page 2 time should be page two");
		help.previous_page();
		assert_eq!(help.page, Page::One, "The page after calling previous_page 3 time should be page one");
	}

	#[test]
	fn render_pagination_test() {
		let mut help = Help::new();
		assert_eq!(
			strip_ansi_border(help.render_pagination().strip_suffix("\n").unwrap()).len(),
			BOARD_WIDTH * 2,
			"The pagination for page one should render the correct length"
		);
		help.next_page();
		assert_eq!(
			strip_ansi_border(help.render_pagination().strip_suffix("\n").unwrap()).len(),
			BOARD_WIDTH * 2,
			"The pagination for page two should render the correct length"
		);
		help.next_page();
		assert_eq!(
			strip_ansi_border(help.render_pagination().strip_suffix("\n").unwrap()).len(),
			BOARD_WIDTH * 2,
			"The pagination for page three should render the correct length"
		);
		help.next_page();
		assert_eq!(
			strip_ansi_border(help.render_pagination().strip_suffix("\n").unwrap()).len(),
			BOARD_WIDTH * 2,
			"The pagination for page one should render the correct length"
		);
	}

	#[test]
	fn general_page_line_length_test() {
		let help = Help::new();
		let output = help.render();

		let lines = output.lines().collect::<Vec<&str>>();
		for (i, line) in lines.iter().enumerate() {
			if i < lines.len() - 1 {
				assert_eq!(strip_ansi_border(line).len(), BOARD_WIDTH * 2, "Line {} should be the correct length", i,);
			}
		}
	}

	#[test]
	fn beast_page_line_length_test() {
		let mut help = Help::new();
		help.next_page();
		let output = help.render();

		let lines = output.lines().collect::<Vec<&str>>();
		for (i, line) in lines.iter().enumerate() {
			if i < lines.len() - 1 {
				assert_eq!(strip_ansi_border(line).len(), BOARD_WIDTH * 2, "Line {i} should be the correct length");
			}
		}
	}

	#[test]
	fn scoring_page_line_length_test() {
		let mut help = Help::new();
		help.previous_page();
		let output = help.render();

		let lines = output.lines().collect::<Vec<&str>>();
		for (i, line) in lines.iter().enumerate() {
			if i < lines.len() - 1 {
				assert_eq!(strip_ansi_border(line).len(), BOARD_WIDTH * 2, "Line {i} should be the correct length");
			}
		}
	}
}
