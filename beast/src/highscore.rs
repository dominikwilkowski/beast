//! this module allows to display paginated highscores in the CLI

use highscore_parser::{Highscores, MAX_NAME_LENGTH, Score};
use reqwest::{blocking, header::CONTENT_TYPE};
use std::{
	env,
	sync::{Arc, Mutex, mpsc::Receiver},
	thread,
	time::Duration,
};

use crate::{
	LOGO, Tile,
	game::{ANSI_BOARD_HEIGHT, ANSI_BOLD, ANSI_FOOTER_HEIGHT, ANSI_FRAME_SIZE},
};

const MAX_SCORES: usize = 100;
const WINDOW_HEIGHT: usize = 28;
const LOADING_POSITION: usize = 13;
const ANSI_RESET_FONT: &str = "\x1B[39m";
const ANSI_RESET_BG: &str = "\x1B[49m";
const ALT_BG: [&str; 2] = [ANSI_RESET_BG, "\x1B[48;5;233m"];

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
	fn new() -> Self {
		let mut screen_array = Vec::with_capacity(112);
		screen_array.extend(LOGO.iter().map(|&s| s.to_string()));
		screen_array.push(format!(
			"\x1b[33m▌\x1b[39m                                            {ANSI_BOLD}HIGHSCORES{ANSI_RESET_FONT}                                              \x1b[33m▐\x1b[39m"
		));
		screen_array.push(String::from("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m"));

		for i in 1..=MAX_SCORES {
			screen_array.push(format!(
			"\x1b[33m▌\x1b[39m      {}  {i:<3}  {ANSI_BOLD}    -{ANSI_RESET_FONT}  ...                                                                      \x1B[0m       \x1b[33m▐\x1b[39m"
		, ALT_BG[i % 2]));
		}

		Self {
			scroll: 0,
			screen_array: Arc::new(Mutex::new(screen_array)),
			state: Arc::new(Mutex::new(State::Loading)),
		}
	}

	pub fn new_loading() -> Self {
		let highscore = Self::new();
		highscore.fetch_data();
		highscore
	}

	pub fn new_idle() -> Self {
		let mut highscore = Self::new();
		highscore.state = Arc::new(Mutex::new(State::Idle));
		highscore
	}

	pub fn handle_enter_name(&mut self, input_listener: &Receiver<u8>, score: u16) -> Option<()> {
		let mut name = String::new();

		println!("{}", Self::render_score_input_screen(name.clone()));

		loop {
			if let Ok(byte) = input_listener.try_recv() {
				match byte as char {
					'\n' => {
						if !name.is_empty() {
							break;
						}
					},
					'\u{7f}' | '\x08' => {
						name.pop();
						println!("{}", Self::render_score_input_screen(name.clone()));
					},
					' ' => {
						name.push(' ');
						println!("{}", Self::render_score_input_screen(name.clone()));
					},
					c @ ('a'..='z'
					| 'A'..='Z'
					| '0'..='9'
					| '!'
					| '@'
					| '#'
					| '$'
					| '%'
					| '^'
					| '&'
					| '*'
					| '('
					| ')'
					| '_'
					| '+'
					| '='
					| '-'
					| ':'
					| ';'
					| '"'
					| '\''
					| '?'
					| '<'
					| '>'
					| '['
					| ']'
					| '{'
					| '}'
					| '|'
					| '\\'
					| '/'
					| ','
					| '.') => {
						if name.len() < MAX_NAME_LENGTH {
							name.push(c);
							println!("{}", Self::render_score_input_screen(name.clone()));
						}
					},
					_ => {},
				}
			}
		}

		self.state = Arc::new(Mutex::new(State::Loading));
		self.render_loading();
		println!("{}", Self::render_loading_screen());
		self.submit_name(&name, score)
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
										Self::inject_score_into_screen_array(&mut screen_array, &data);
										if *state == State::Loading {
											*state = State::Idle;
											println!("{}", Self::render_score(screen_array.clone(), scroll_clone));
										}
									},
									Err(error) => {
										if *state == State::Loading {
											*state = State::Error;
											Self::render_error(format!("Failed to parse highscores file: {error}"));
										}
									},
								}
							};
						}
					},
					Err(error) => {
						if let Ok(mut state) = state_clone.lock() {
							if *state == State::Loading {
								*state = State::Error;
								Self::render_error(format!("Error reading highscore data: {error}"));
							}
						}
					},
				},
				Err(error) => {
					if let Ok(mut state) = state_clone.lock() {
						if *state == State::Loading {
							*state = State::Error;
							Self::render_error(format!("Fetching highscore failed: {error}"));
						}
					}
				},
			}
		});
	}

	pub fn submit_name(&self, name: &str, score: u16) -> Option<()> {
		let state_clone = Arc::clone(&self.state);
		let name_clone = name.to_string();

		let mut url = env::var("HIGHSCORE_URL").unwrap_or(String::from("https://dominik-wilkowski.com/beast"));
		url.push_str("/highscore");

		match Highscores::ron_to_str(&Score {
			name: name_clone,
			score,
		}) {
			Ok(payload) => {
				match blocking::Client::new().post(&url).header(CONTENT_TYPE, "application/x-ron").body(payload).send() {
					Ok(response) => {
						if let Ok(mut state) = state_clone.lock() {
							if *state == State::Loading {
								if response.status().is_success() {
									*state = State::Idle;
									Some(())
								} else {
									*state = State::Error;
									let error = response.text().unwrap_or_else(|_| "Could not read error response".to_string());
									Self::render_error(format!("Failed to post highscore: {error}"));
									None
								}
							} else {
								None
							}
						} else {
							None
						}
					},
					Err(error) => {
						if let Ok(mut state) = state_clone.lock() {
							if *state == State::Loading {
								*state = State::Error;
								Self::render_error(format!("Failed to parse highscores file: {error}"));
							}
						}
						None
					},
				}
			},
			Err(error) => {
				if let Ok(mut state) = state_clone.lock() {
					if *state == State::Loading {
						*state = State::Error;
						Self::render_error(format!("Failed to parse highscores file: {error}"));
					}
				}
				None
			},
		}
	}

	fn inject_score_into_screen_array(screen_array: &mut [String], data: &Highscores) {
		for (index, score) in data.scores.iter().enumerate() {
			screen_array[index + 12] = format!(
				"\x1b[33m▌\x1b[39m      {}  {:<3}  {ANSI_BOLD}{:>5}{ANSI_RESET_FONT}  {:<50}  \x1B[38;5;239m{:<19}{ANSI_RESET_FONT}  {ANSI_RESET_BG}       \x1b[33m▐\x1b[39m",
				ALT_BG[(index + 1) % 2],
				index + 1,
				score.score,
				score.name,
				score.format_timestamp()
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
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m                                  {ANSI_BOLD}[SPACE]{ANSI_RESET_FONT} Play  {ANSI_BOLD}[Q]{ANSI_RESET_FONT} Quit  {ANSI_BOLD}[H]{ANSI_RESET_FONT} Help                                  \x1b[33m▐\x1b[39m\n"));
		output.push_str(&bottom_pos);

		output
	}

	pub fn render_score_input_screen(name: String) -> String {
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
		output.push_str("\x1b[33m▌\x1b[39m                                        Enter your name below                                       \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                        ┌──────────────────────────────────────────────────┐                        \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!(
			"\x1b[33m▌\x1b[39m                        │{name:<50}│                        \x1b[33m▐\x1b[39m\n"
		));
		output.push_str("\x1b[33m▌\x1b[39m                        └──────────────────────────────────────────────────┘                        \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m                                        {ANSI_BOLD}[ENTER]{ANSI_RESET_FONT} Submit score                                        \x1b[33m▐\x1b[39m\n"));
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
				std::thread::sleep(Duration::from_millis(100));
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
		output.push_str(&format!("\x1b[33m▌\x1b[39m            {ANSI_BOLD}[SPACE]{ANSI_RESET_FONT} Play  {ANSI_BOLD}[Q]{ANSI_RESET_FONT} Quit  {ANSI_BOLD}[H]{ANSI_RESET_FONT} Help  {ANSI_BOLD}[↓]{ANSI_RESET_FONT} Scroll Down  {ANSI_BOLD}[↑]{ANSI_RESET_FONT} Scroll Up  {ANSI_BOLD}[R]{ANSI_RESET_FONT} Refresh           \x1b[33m▐\x1b[39m\n"));
		output.push_str(&bottom_pos);

		output
	}
}
