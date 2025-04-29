//! this module allows to display paginated highscores and publish new scores

use reqwest::{blocking, header::CONTENT_TYPE};
use std::{
	env,
	sync::{Arc, Mutex, mpsc::Receiver},
	thread,
	time::Duration,
};

use crate::{
	ANSI_BOLD, ANSI_LEFT_BORDER, ANSI_RESET, ANSI_RESET_BG, ANSI_RESET_FONT, ANSI_RIGHT_BORDER, LOGO, Tile,
	common::{Highscores, MAX_NAME_LENGTH, MAX_SCORES, Score, levels::Level},
	game::{ANSI_BOARD_HEIGHT, ANSI_FOOTER_HEIGHT, ANSI_FRAME_SIZE},
};

/// the height of the window
const WINDOW_HEIGHT: usize = 28;
/// where the loading is displayed
const LOADING_POSITION: usize = 13;
/// the two types of backgrounds displayed in the score table
const ALT_BG: [&str; 2] = [ANSI_RESET_BG, "\x1B[48;5;233m"];

/// the state our highscore can be in
#[derive(Debug, Clone, PartialEq)]
pub enum State {
	/// we're loading the data from the server
	Loading,
	/// we're displaying the highscore
	Idle,
	/// we encountered an error
	Error,
	/// we end the highscore
	Quit,
}

/// the highscore fetches its own data from the server and keeps track of it's scroll position
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
			"{ANSI_LEFT_BORDER}                                            {ANSI_BOLD}HIGHSCORES{ANSI_RESET}                                              {ANSI_RIGHT_BORDER}"
		));
		screen_array.push(format!(
			"{ANSI_LEFT_BORDER}     \x1B[38;5;241mPOS  SCORE  NAME                                                LEVEL  DATE{ANSI_RESET_FONT}                    {ANSI_RIGHT_BORDER}",
		));
		screen_array.push(format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}"));

		for i in 1..=MAX_SCORES {
			let bg = ALT_BG[(i + 1) % 2];
			screen_array.push(format!(
			"{ANSI_LEFT_BORDER}   {bg}  {i:<3}  {ANSI_BOLD}    -{ANSI_RESET}{bg}  ...                                                                             {ANSI_RESET_BG}   {ANSI_RIGHT_BORDER}"));
		}

		Self {
			scroll: 0,
			screen_array: Arc::new(Mutex::new(screen_array)),
			state: Arc::new(Mutex::new(State::Loading)),
		}
	}

	/// create a new instance of highscore and default to a loading state
	pub fn new_loading() -> Self {
		let highscore = Self::new();
		highscore.fetch_data();
		highscore
	}

	/// create a new instance of highscore and default to an idle state
	pub fn new_idle() -> Self {
		let highscore = Self::new();
		*highscore.state.lock().unwrap() = State::Idle;
		highscore
	}

	/// listen to stdin to capture the name you want to enter into the highscore
	pub fn handle_enter_name(&mut self, input_listener: &Receiver<u8>, score: u16, level: Level) -> Option<()> {
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

		*self.state.lock().unwrap() = State::Loading;
		self.render_loading();
		println!("{}", Self::render_loading_screen());
		self.submit_name(&name, score, level)
	}

	/// scroll down
	pub fn scroll_down(&mut self) {
		self.scroll = if self.scroll >= 85 { 85 } else { self.scroll + 1 };
	}

	/// scroll up
	pub fn scroll_up(&mut self) {
		self.scroll = if self.scroll == 0 { 0 } else { self.scroll - 1 };
	}

	/// render the highscore while taking into account the scroll position
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

	/// fetch the data from the server
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
											println!(
												"{}{}",
												Self::render_loading_screen(),
												Self::render_error(format!("Failed to parse highscores file: {error}"))
											);
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
								println!(
									"{}{}",
									Self::render_loading_screen(),
									Self::render_error(format!("Error reading highscore data: {error}"))
								);
							}
						}
					},
				},
				Err(error) => {
					if let Ok(mut state) = state_clone.lock() {
						if *state == State::Loading {
							*state = State::Error;
							println!(
								"{}{}",
								Self::render_loading_screen(),
								Self::render_error(format!("Fetching highscore failed: {error}"))
							);
						}
					}
				},
			}
		});
	}

	fn submit_name(&self, name: &str, score: u16, level: Level) -> Option<()> {
		let state_clone = Arc::clone(&self.state);
		let name_clone = name.to_string();

		let mut url = env::var("HIGHSCORE_URL").unwrap_or(String::from("https://dominik-wilkowski.com/beast"));
		url.push_str("/highscore");

		match Highscores::ron_to_str(&Score {
			name: name_clone,
			score,
			level,
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
									let error = response.text().unwrap_or_else(|_| String::from("Could not read error response"));
									println!(
										"{}{}",
										Self::render_loading_screen(),
										Self::render_error(format!("Failed to post highscore: {error}"))
									);
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
								println!(
									"{}{}",
									Self::render_loading_screen(),
									Self::render_error(format!("Failed to parse highscores file: {error}"))
								);
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
						println!(
							"{}{}",
							Self::render_loading_screen(),
							Self::render_error(format!("Failed to parse highscores file: {error}"))
						);
					}
				}
				None
			},
		}
	}

	fn inject_score_into_screen_array(screen_array: &mut [String], data: &Highscores) {
		for (index, score) in data.scores.iter().enumerate() {
			let bg = ALT_BG[index % 2];
			screen_array[index + 13] = format!(
				"{ANSI_LEFT_BORDER}   {bg}  {:<3}  {ANSI_BOLD}{:>5}{ANSI_RESET}{bg}  {:<50}  {:<5}  \x1B[38;5;239m{:<19}{ANSI_RESET_FONT}  {ANSI_RESET_BG}   {ANSI_RIGHT_BORDER}",
				index + 1,
				score.score,
				score.name,
				score.level.to_string(),
				score.format_timestamp(),
			);
		}
	}

	/// render the loading screen
	pub fn render_loading_screen() -> String {
		let mut output = String::new();
		let top_pos = format!("\x1b[{}F", ANSI_FRAME_SIZE + ANSI_BOARD_HEIGHT + ANSI_FRAME_SIZE + ANSI_FOOTER_HEIGHT);
		let bottom_pos = format!("\x1b[{}E", ANSI_FRAME_SIZE + ANSI_FOOTER_HEIGHT);

		output.push_str(&top_pos);
		output.push_str(&LOGO.join("\n"));
		output.push('\n');
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                  {ANSI_BOLD}[SPACE]{ANSI_RESET} Play  {ANSI_BOLD}[Q]{ANSI_RESET} Quit  {ANSI_BOLD}[H]{ANSI_RESET} Help                                  {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&bottom_pos);

		output
	}

	fn render_score_input_screen(name: String) -> String {
		let mut output = String::new();
		let top_pos = format!("\x1b[{}F", ANSI_FRAME_SIZE + ANSI_BOARD_HEIGHT + ANSI_FRAME_SIZE + ANSI_FOOTER_HEIGHT);
		let bottom_pos = format!("\x1b[{}E", ANSI_FRAME_SIZE + ANSI_FOOTER_HEIGHT);

		output.push_str(&top_pos);
		output.push_str(&LOGO.join("\n"));
		output.push('\n');
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                        Enter your name below                                       {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                        ┌──────────────────────────────────────────────────┐                        {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!(
			"{ANSI_LEFT_BORDER}                        │{name:<50}│                        {ANSI_RIGHT_BORDER}\n"
		));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                        └──────────────────────────────────────────────────┘                        {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                        {ANSI_BOLD}[ENTER]{ANSI_RESET} Submit score                                        {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&bottom_pos);

		output
	}

	/// render the loading animaiton in a separate thread
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
					"{top_pos}{ANSI_LEFT_BORDER}                                               LOADING                                              {ANSI_RIGHT_BORDER}"
				);
				println!(
					"{ANSI_LEFT_BORDER}                                            {:>12}                                            {ANSI_RIGHT_BORDER}{bottom_pos}",
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

	fn render_error(mut error: String) -> String {
		let top_pos = format!("\x1b[{}F", LOADING_POSITION + ANSI_FRAME_SIZE + ANSI_FOOTER_HEIGHT + 1);
		let bottom_pos = format!("\x1b[{}E", LOADING_POSITION + ANSI_FRAME_SIZE + ANSI_FOOTER_HEIGHT + 1);
		error.truncate(98);
		format!("{top_pos}{ANSI_LEFT_BORDER}{error:^100}{ANSI_RESET}{ANSI_RIGHT_BORDER}{bottom_pos}")
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
		output.push_str(&format!("{ANSI_LEFT_BORDER}                                                                                                    {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&format!("{ANSI_LEFT_BORDER}            {ANSI_BOLD}[SPACE]{ANSI_RESET} Play  {ANSI_BOLD}[Q]{ANSI_RESET} Quit  {ANSI_BOLD}[H]{ANSI_RESET} Help  {ANSI_BOLD}[↓]{ANSI_RESET} Scroll Down  {ANSI_BOLD}[↑]{ANSI_RESET} Scroll Up  {ANSI_BOLD}[R]{ANSI_RESET} Refresh           {ANSI_RIGHT_BORDER}\n"));
		output.push_str(&bottom_pos);

		output
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::{BOARD_WIDTH, test_common::strip_ansi_border};

	#[test]
	fn initial_state_test() {
		let highscore_idle = Highscore::new_idle();
		assert_eq!(*highscore_idle.state.lock().unwrap(), State::Idle, "The initial state from new_idle should be Idle");
	}

	#[test]
	fn scroll_test() {
		let mut highscore = Highscore::new_idle();
		assert_eq!(highscore.scroll, 0, "Initial scroll should be 0");

		highscore.scroll_down();
		assert_eq!(highscore.scroll, 1, "After scrolling down once, scroll should be 1");

		highscore.scroll_up();
		assert_eq!(highscore.scroll, 0, "After scrolling up, scroll should be 0");

		highscore.scroll_up();
		assert_eq!(highscore.scroll, 0, "Scrolling up at minimum should remain at 0");

		for _ in 0..100 {
			highscore.scroll_down();
		}
		assert_eq!(highscore.scroll, 85, "Scroll should cap at 85");
	}

	#[test]
	fn screen_array_initialization_test() {
		let highscore = Highscore::new_idle();
		let screen_array = highscore.screen_array.lock().unwrap().clone();

		for (i, logo_line) in LOGO.iter().enumerate() {
			assert_eq!(screen_array[i], logo_line.to_string(), "Logo line {i} should be copied correctly");
		}

		assert!(screen_array[LOGO.len()].contains("HIGHSCORES"), "Title should contain HIGHSCORES");
		assert_eq!(screen_array.len(), LOGO.len() + 3 + MAX_SCORES, "Screen array should have the correct number of lines");
		assert!(screen_array[LOGO.len() + 3].contains("  1  "), "First score placeholder should have index 1");
		assert!(screen_array[LOGO.len() + 3].contains("-"), "First score placeholder should have a dash");
	}

	#[test]
	fn render_loading_screen_line_length_test() {
		let output = Highscore::render_loading_screen();

		let lines = output.lines().collect::<Vec<&str>>();
		for (i, line) in lines.iter().enumerate() {
			if i < lines.len() - 1 {
				assert_eq!(strip_ansi_border(line).len(), BOARD_WIDTH * 2, "Line {i} should be the correct length");
			}
		}
	}

	#[test]
	fn render_score_input_screen_line_length_test() {
		let name = String::from("TestPlayer");
		let output = Highscore::render_score_input_screen(name);

		let lines = output.lines().collect::<Vec<&str>>();
		for (i, line) in lines.iter().enumerate() {
			if i < lines.len() - 1 {
				assert_eq!(strip_ansi_border(line).len(), BOARD_WIDTH * 2, "Line {i} should be the correct length");
			}
		}
	}

	#[test]
	fn render_score_input_screen_name_display_test() {
		let empty_name = "".to_string();
		let output_empty = Highscore::render_score_input_screen(empty_name);
		assert!(output_empty.contains("│                                                  │"), "Input box should be empty");

		let name = "TestPlayer".to_string();
		let output = Highscore::render_score_input_screen(name);
		assert!(
			output.contains("│TestPlayer                                        │"),
			"Input box should contain the name"
		);

		let max_name = "X".repeat(MAX_NAME_LENGTH);
		let output_max = Highscore::render_score_input_screen(max_name);
		assert!(
			output_max.contains(&format!("│{:<50}│", "X".repeat(MAX_NAME_LENGTH))),
			"Input box should contain the full max-length name"
		);
	}

	#[test]
	fn render_score_line_length_test() {
		let highscore = Highscore::new_idle();
		let screen_array = highscore.screen_array.lock().unwrap().clone();

		let output = Highscore::render_score(screen_array, 0);

		let lines = output.lines().collect::<Vec<&str>>();
		for (i, line) in lines.iter().enumerate() {
			if i < lines.len() - 1 {
				assert_eq!(strip_ansi_border(line).len(), BOARD_WIDTH * 2, "Line {i} should be the correct length");
			}
		}
	}

	#[test]
	fn render_score_scroll_test() {
		let highscore = Highscore::new_idle();
		let mut screen_array = highscore.screen_array.lock().unwrap().clone();

		Highscore::inject_score_into_screen_array(
			&mut screen_array,
			&Highscores {
				scores: vec![
					crate::common::Highscore::new("Dom", 666, Level::One),
					crate::common::Highscore::new("Belle", 42, Level::Two),
				],
			},
		);
		*highscore.screen_array.lock().unwrap() = screen_array.clone();

		let output_0 = Highscore::render_score(screen_array.clone(), 0);
		assert!(output_0.contains("Dom"), "First score should be visible with scroll = 0");

		let output_14 = Highscore::render_score(screen_array.clone(), 14);
		assert!(!output_14.contains("Dom"), "First score should not be visible with scroll = 14");
		assert!(output_14.contains("Belle"), "Second score should be visible with scroll = 14");
	}

	#[test]
	fn state_rendering_test() {
		let highscore = Highscore::new_idle();

		assert!(!highscore.render().is_empty(), "Idle state should render content");

		*highscore.state.lock().unwrap() = State::Error;
		assert!(highscore.render().is_empty(), "Error state should render empty string");

		*highscore.state.lock().unwrap() = State::Quit;
		assert!(highscore.render().is_empty(), "Quit state should render empty string");
	}

	#[test]
	fn render_method_dispatches_correctly_test() {
		let highscore = Highscore::new_idle();

		let idle_render = highscore.render();
		assert!(!idle_render.is_empty(), "Idle state should render content");
		assert!(idle_render.contains("Scroll Down"), "Idle state should render score screen with scroll controls");

		*highscore.state.lock().unwrap() = State::Loading;
		let loading_render = highscore.render();
		assert!(!loading_render.contains("LOADING"), "Loading state should render empty screen");
	}

	#[test]
	fn inject_score_into_screen_array_test() {
		let mut screen_array = Vec::new();
		for _ in 0..LOGO.len() + 2 + MAX_SCORES {
			screen_array.push(String::new());
		}

		Highscore::inject_score_into_screen_array(
			&mut screen_array,
			&Highscores {
				scores: vec![
					crate::common::Highscore::new("Player 1", 100, Level::Six),
					crate::common::Highscore::new("Player 2", 200, Level::Eight),
				],
			},
		);

		let first_score_line = &screen_array[LOGO.len() + 3];
		let second_score_line = &screen_array[LOGO.len() + 4];

		assert!(first_score_line.contains("Player 1"), "First score line should contain Player 1");
		assert!(first_score_line.contains("100"), "First score line should contain score 100");
		assert!(first_score_line.contains("6"), "First score line should contain level six");
		assert!(second_score_line.contains("Player 2"), "Second score line should contain Player 2");
		assert!(second_score_line.contains("200"), "Second score line should contain score 200");
		assert!(second_score_line.contains("8"), "Second score line should contain level eight");
	}

	#[test]
	fn render_error_test() {
		let error = String::from("Short error message");
		let error_output = Highscore::render_error(error.clone());
		let line = error_output.lines().next().unwrap();
		assert_eq!(strip_ansi_border(line).len(), BOARD_WIDTH * 2, "Short error line should have the correct length");
		assert!(error_output.contains(&error), "Output should contain the original short error message");

		let error = "X".repeat(98);
		let error_output = Highscore::render_error(error.clone());
		let line = error_output.lines().next().unwrap();
		assert_eq!(
			strip_ansi_border(line).len(),
			BOARD_WIDTH * 2,
			"Exact-length error line should have the correct length"
		);
		assert!(error_output.contains(&error), "Output should contain the 98-character error message without truncation");

		let mut error = "X".repeat(98);
		error.push_str("OOOOOOO");
		let error_output = Highscore::render_error(error.clone());
		let line = error_output.lines().next().unwrap();
		assert_eq!(strip_ansi_border(line).len(), BOARD_WIDTH * 2, "Truncated error line should have the correct length");
		assert!(error_output.contains(&error[0..98].to_string()), "Output should contain the truncated error message");
		assert!(!error_output.contains("OOOOOOO"), "Output should not contain the truncated portion of the error");

		let output = Highscore::render_error(String::from(""));
		assert!(
			output.contains(ANSI_LEFT_BORDER) && output.contains(ANSI_RIGHT_BORDER),
			"Empty error should still have proper formatting"
		);
	}
}
