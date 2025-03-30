//! this module allows to display paginated highscores in the CLI
use crate::{
	LOGO,
	game::{ANSI_BOARD_HEIGHT, ANSI_BOLD, ANSI_FOOTER_HEIGHT, ANSI_FRAME_SIZE, ANSI_RESET},
};

const MAX_SCORES: usize = 100;
const WINDOW_HEIGHT: usize = 28;

enum State {
	Loading,
	Idle,
}

pub struct Highscore {
	scroll: usize,
	screen_array: Vec<String>,
	state: State,
}

impl Highscore {
	pub fn new() -> Self {
		let mut screen_array = Vec::with_capacity(112);
		screen_array.extend(LOGO.iter().map(|&s| s.to_string()));
		screen_array.push(format!(
			"\x1b[33m▌\x1b[39m                                            {ANSI_BOLD}HIGHSCORES{ANSI_RESET}                                              \x1b[33m▐\x1b[39m"
		));
		screen_array.push(String::from("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m"));

		for i in 1..=MAX_SCORES {
			screen_array.push(format!(
			"\x1b[33m▌\x1b[39m                   {i:<3}  {ANSI_BOLD}00000{ANSI_RESET}  ...                                                                  \x1b[33m▐\x1b[39m"
		));
		}

		Self {
			scroll: 0,
			screen_array,
			state: State::Idle,
		}
	}

	pub fn scroll_down(&mut self) {
		self.scroll = if self.scroll >= 84 { 84 } else { self.scroll + 1 };
	}

	pub fn scroll_up(&mut self) {
		self.scroll = if self.scroll == 0 { 0 } else { self.scroll - 1 };
	}

	pub fn render(&self) -> String {
		match self.state {
			State::Loading => String::from("Loading..."), // TODO
			State::Idle => self.render_screen(),
		}
	}

	pub fn fetch_data(&mut self) {
		// TODO
	}

	fn render_screen(&self) -> String {
		let mut output = String::new();
		let top_pos = format!("\x1b[{}F", ANSI_BOARD_HEIGHT + ANSI_FRAME_SIZE + ANSI_FOOTER_HEIGHT + 1);
		let bottom_pos = format!("\x1b[{}E", ANSI_FRAME_SIZE + ANSI_FOOTER_HEIGHT);

		let start = self.scroll;
		let end = (self.scroll + WINDOW_HEIGHT).min(self.screen_array.len());

		output.push_str(&top_pos);
		output.push_str(&self.screen_array[start..end].join("\n"));
		output.push_str("\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m            {ANSI_BOLD}[SPACE]{ANSI_RESET} Play  {ANSI_BOLD}[Q]{ANSI_RESET} Quit  {ANSI_BOLD}[H]{ANSI_RESET} Help  {ANSI_BOLD}[↓]{ANSI_RESET} Scroll Down  {ANSI_BOLD}[↑]{ANSI_RESET} Scroll Up  {ANSI_BOLD}[R]{ANSI_RESET} Refresh           \x1b[33m▐\x1b[39m\n"));
		output.push_str(&bottom_pos);

		output
	}
}
