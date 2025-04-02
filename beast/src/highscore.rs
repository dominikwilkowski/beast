//! this module allows to display paginated highscores in the CLI

use highscore_parser::{Highscores, Score};
use reqwest::{blocking, header::CONTENT_TYPE};
use std::{
	env,
	error::Error,
	sync::{Arc, Mutex},
	thread,
};

use crate::{
	LOGO, Tile,
	game::{ANSI_BOARD_HEIGHT, ANSI_BOLD, ANSI_FOOTER_HEIGHT, ANSI_FRAME_SIZE, ANSI_RESET},
};

const MAX_SCORES: usize = 100;
const WINDOW_HEIGHT: usize = 28;
const LOADING_POSITION: usize = 13;

#[derive(Debug, Clone, PartialEq)]
pub enum State {
	Loading,
	Idle,
	Error,
	Quit,
}

pub struct Highscore {
	scroll: usize,
	screen_array: Arc<Mutex<Vec<String>>>,
	pub state: Arc<Mutex<State>>,
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
			"\x1b[33m▌\x1b[39m                   {i:<3}  {ANSI_BOLD}    -{ANSI_RESET}  ...                                                                  \x1b[33m▐\x1b[39m"
		));
		}

		let highscore = Self {
			scroll: 0,
			screen_array: Arc::new(Mutex::new(screen_array)),
			state: Arc::new(Mutex::new(State::Loading)),
		};

		highscore.fetch_data();

		highscore
	}

	pub fn scroll_down(&mut self) {
		self.scroll = if self.scroll >= 84 { 84 } else { self.scroll + 1 };
	}

	pub fn scroll_up(&mut self) {
		self.scroll = if self.scroll == 0 { 0 } else { self.scroll - 1 };
	}

	pub fn render(&self) -> String {
		let state = self.state.lock().unwrap();
		let screen_array = self.screen_array.lock().unwrap();
		match *state {
			State::Loading => {
				self.render_loading();
				Self::render_loading_screen()
			},
			State::Idle => Self::render_score(screen_array.clone(), self.scroll),
			State::Error => String::new(),
			State::Quit => String::new(),
		}
	}

	pub fn fetch_data(&self) {
		let state_clone = Arc::clone(&self.state);
		let screen_array_clone = Arc::clone(&self.screen_array);
		let scroll_clone = self.scroll;

		thread::spawn(move || {
			let mut url = env::var("HIGHSCORE_URL").unwrap_or(String::from("https://dominik-wilkowski.com/beast"));
			url.push_str("/highscore");

			match blocking::get(url) {
				Ok(responds) => match responds.text() {
					Ok(body) => {
						if let Ok(mut state) = state_clone.lock() {
							if let Ok(mut screen_array) = screen_array_clone.lock() {
								match Highscores::ron_from_str(&body) {
									Ok(data) => {
										Self::enter_score(&mut screen_array, &data);
										*state = State::Idle;
										println!("{}", Self::render_score(screen_array.clone(), scroll_clone));
									},
									Err(error) => {
										*state = State::Error;
										Self::render_error(format!("Failed to parse highscores file: {error}"));
									},
								}
							};
						}
					},
					Err(error) => {
						if let Ok(mut state) = state_clone.lock() {
							*state = State::Error;
							Self::render_error(format!("Error reading highscore data: {error}"));
						}
					},
				},
				Err(error) => {
					if let Ok(mut state) = state_clone.lock() {
						*state = State::Error;
						Self::render_error(format!("Fetching highscore failed: {error}"));
					}
				},
			}
		});
	}

	pub fn enter_name(name: &str, score: u16) -> Result<(), Box<dyn Error>> {
		let mut url = env::var("HIGHSCORE_URL").unwrap_or(String::from("https://dominik-wilkowski.com/beast"));
		url.push_str("/highscore");

		let payload = Highscores::ron_to_str(&Score {
			name: name.to_string(),
			score,
		})?;
		let response = blocking::Client::new().post(&url).header(CONTENT_TYPE, "application/x-ron").body(payload).send()?;

		if response.status().is_success() {
			Ok(())
		} else {
			let error = response.text().unwrap_or_else(|_| "Could not read error response".to_string());
			Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to post highscore: {error}"))))
		}
	}

	fn enter_score(screen_array: &mut [String], data: &Highscores) {
		for (index, score) in data.scores.iter().enumerate() {
			screen_array[index + 12] = format!(
				"\x1b[33m▌\x1b[39m                   {:<3}  {ANSI_BOLD}{:>5}{ANSI_RESET}  {:<50}                   \x1b[33m▐\x1b[39m",
				index + 1,
				score.score,
				score.name
			);
		}
	}

	pub fn render_loading_screen() -> String {
		let mut output = String::new();
		let top_pos = format!("\x1b[{}F", ANSI_FRAME_SIZE + ANSI_BOARD_HEIGHT + ANSI_FRAME_SIZE + ANSI_FOOTER_HEIGHT);
		let bottom_pos = format!("\x1b[{}E", ANSI_FRAME_SIZE + ANSI_FOOTER_HEIGHT);

		output.push_str(&top_pos);
		output.push_str(&LOGO.join("\n"));
		output.push('\n');
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
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m                            {ANSI_BOLD}[SPACE]{ANSI_RESET} Play  {ANSI_BOLD}[Q]{ANSI_RESET} Quit  {ANSI_BOLD}[H]{ANSI_RESET} Help  {ANSI_BOLD}[R]{ANSI_RESET} Refresh                           \x1b[33m▐\x1b[39m\n"));
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&bottom_pos);

		output
	}

	pub fn render_loading(&self) {
		let state_clone = Arc::clone(&self.state);

		thread::spawn(move || {
			let player = Tile::Player;
			let block = Tile::Block;
			let beast = Tile::CommonBeast;
			let loading_frames = [
				format!("{player}    {block}{beast}{block}"),
				format!("  {player}  {block}{beast}{block}"),
				format!("    {player}{block}{beast}{block}"),
				format!("      {player}{block}{block}"),
				format!("        {player}{block}"),
				format!("          {player}"),
				format!("{block}{beast}{block}    {player}"),
				format!("{block}{beast}{block}  {player}  "),
				format!("{block}{beast}{block}{player}    "),
				format!("{block}{block}{player}      "),
				format!("{block}{player}        "),
				format!("{player}          "),
			];
			let mut frame_index = 0;

			while *state_clone.lock().unwrap() == State::Loading {
				let top_pos = format!("\x1b[{}F", LOADING_POSITION + ANSI_FRAME_SIZE + ANSI_FOOTER_HEIGHT + 2);
				let bottom_pos = format!("\x1b[{}E", LOADING_POSITION + ANSI_FRAME_SIZE + ANSI_FOOTER_HEIGHT);
				println!(
					"{top_pos}\x1b[33m▌\x1b[39m                                               LOADING                                              \x1b[33m▐\x1b[39m"
				);
				println!(
					"\x1b[33m▌\x1b[39m                                            {:>12}                                            \x1b[33m▐\x1b[39m{bottom_pos}",
					loading_frames[frame_index]
				);
				frame_index += 1;
				if frame_index >= loading_frames.len() {
					frame_index = 0;
				}
				std::thread::sleep(std::time::Duration::from_millis(100));
			}
		});
	}

	fn render_error(mut error: String) {
		println!("{}", Self::render_loading_screen());
		let top_pos = format!("\x1b[{}F", LOADING_POSITION + ANSI_FRAME_SIZE + ANSI_FOOTER_HEIGHT + 1);
		let bottom_pos = format!("\x1b[{}E", LOADING_POSITION + ANSI_FRAME_SIZE + ANSI_FOOTER_HEIGHT);
		error.truncate(98);
		println!("{top_pos}\x1b[33m▌\x1b[39m{error:^100}\x1b[33m▐\x1b[39m{bottom_pos}");
	}

	fn render_score(screen_array: Vec<String>, scroll: usize) -> String {
		let mut output = String::new();
		let top_pos = format!("\x1b[{}F", ANSI_BOARD_HEIGHT + ANSI_FRAME_SIZE + ANSI_FOOTER_HEIGHT + 1);
		let bottom_pos = format!("\x1b[{}E", ANSI_FRAME_SIZE + ANSI_FOOTER_HEIGHT);
		let start = scroll;
		let end = (scroll + WINDOW_HEIGHT).min(screen_array.len());

		output.push_str(&top_pos);
		output.push_str(&screen_array[start..end].join("\n"));
		output.push('\n');
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m            {ANSI_BOLD}[SPACE]{ANSI_RESET} Play  {ANSI_BOLD}[Q]{ANSI_RESET} Quit  {ANSI_BOLD}[H]{ANSI_RESET} Help  {ANSI_BOLD}[↓]{ANSI_RESET} Scroll Down  {ANSI_BOLD}[↑]{ANSI_RESET} Scroll Up  {ANSI_BOLD}[R]{ANSI_RESET} Refresh           \x1b[33m▐\x1b[39m\n"));
		output.push_str(&bottom_pos);

		output
	}
}
