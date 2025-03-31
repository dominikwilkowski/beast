//! this module contains the main struct that orchestrates the game

use std::{
	io::{self, Read},
	sync::mpsc,
	thread,
	time::{Duration, Instant},
};

use crate::{
	BOARD_HEIGHT, BOARD_WIDTH, Dir, LOGO, Tile,
	beasts::{Beast, BeastAction, CommonBeast, Egg, HatchedBeast, HatchingState, SuperBeast},
	board::Board,
	help::Help,
	highscore::{Highscore, State},
	levels::Level,
	player::{Player, PlayerAction},
	raw_mode::{RawMode, install_raw_mode_signal_handler},
};

pub const ANSI_BOARD_HEIGHT: usize = BOARD_HEIGHT;
pub const ANSI_FRAME_SIZE: usize = 1;
pub const ANSI_FOOTER_HEIGHT: usize = 2;
pub const ANSI_BOLD: &str = "\x1B[1m";
pub const ANSI_RESET: &str = "\x1B[0m";
const TICK_DURATION: Duration = Duration::from_millis(200);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Beat {
	One,
	Two,
	Three,
	Four,
	Five,
}

impl Beat {
	pub fn next(&self) -> Self {
		match self {
			Self::One => Self::Two,
			Self::Two => Self::Three,
			Self::Three => Self::Four,
			Self::Four => Self::Five,
			Self::Five => Self::One,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
	Intro,
	Playing,
	Dying(Beat),
	Killing(Beat),
	Help,
	HighScore,
	GameOver,
	Won,
	Quit,
}

pub struct Game {
	pub board: Board,
	pub level: Level,
	pub level_start: Instant,
	pub common_beasts: Vec<CommonBeast>,
	pub super_beasts: Vec<SuperBeast>,
	pub eggs: Vec<Egg>,
	pub hatched_beasts: Vec<HatchedBeast>,
	pub player: Player,
	pub state: GameState,
	beat: Beat,
	input_listener: mpsc::Receiver<u8>,
	_raw_mode: RawMode,
}

impl Game {
	pub fn new() -> Self {
		let board_terrain_info = Board::generate_terrain(Level::One);

		install_raw_mode_signal_handler();
		let _raw_mode = RawMode::enter().unwrap_or_else(|error| {
			eprintln!("Raw mode could not be entered in this shell: {error}",);
			std::process::exit(1);
		});
		let (sender, receiver) = mpsc::channel::<u8>();
		{
			let stdin = io::stdin();
			thread::spawn(move || {
				let mut lock = stdin.lock();
				let mut buffer = [0u8; 1];
				while lock.read_exact(&mut buffer).is_ok() {
					if sender.send(buffer[0]).is_err() {
						break;
					}
				}
			});
		}

		Self {
			board: Board::new(board_terrain_info.data),
			level: Level::One,
			level_start: Instant::now(),
			common_beasts: board_terrain_info.common_beasts,
			super_beasts: board_terrain_info.super_beasts,
			eggs: board_terrain_info.eggs,
			hatched_beasts: board_terrain_info.hatched_beasts,
			player: board_terrain_info.player,
			state: GameState::Intro,
			beat: Beat::One,
			input_listener: receiver,
			_raw_mode,
		}
	}

	pub fn play(&mut self) {
		let last_tick = Instant::now();

		loop {
			match self.state {
				GameState::Intro => {
					self.handle_intro_state();
				},
				GameState::Playing | GameState::Dying(_) | GameState::Killing(_) => {
					self.handle_playing_state(last_tick);
				},
				GameState::Help => {
					self.handle_help_state();
				},
				GameState::HighScore => {
					self.handle_highscore_state();
				},
				GameState::GameOver => {
					self.handle_death_state();
				},
				GameState::Won => {
					self.handle_win_state();
				},
				GameState::Quit => {
					println!("Bye...");
					break;
				},
			}
		}
	}

	fn handle_intro_state(&mut self) {
		println!("{}", Self::render_intro());

		loop {
			if let Ok(byte) = self.input_listener.try_recv() {
				match byte as char {
					' ' => {
						self.level_start = Into::into(Instant::now());
						self.state = GameState::Playing;
						break;
					},
					'h' | 'H' => {
						self.level_start = Into::into(Instant::now());
						self.state = GameState::Help;
						break;
					},
					's' | 'S' => {
						self.state = GameState::HighScore;
						break;
					},
					'q' | 'Q' => {
						self.state = GameState::Quit;
						break;
					},
					_ => {},
				}
			}
		}
	}

	fn handle_playing_state(&mut self, mut last_tick: Instant) {
		print!("{}", self.render_board());

		loop {
			if let Ok(byte) = self.input_listener.try_recv() {
				if byte == 0x1B {
					let second = self.input_listener.recv().unwrap_or(0);
					let third = self.input_listener.recv().unwrap_or(0);
					if second == b'[' {
						let mut render = false;
						let player_action = match third {
							b'A' => {
								let player_action = self.player.advance(&mut self.board, &Dir::Up);
								render = true;
								player_action
							},
							b'C' => {
								let player_action = self.player.advance(&mut self.board, &Dir::Right);
								render = true;
								player_action
							},
							b'B' => {
								let player_action = self.player.advance(&mut self.board, &Dir::Down);
								render = true;
								player_action
							},
							b'D' => {
								let player_action = self.player.advance(&mut self.board, &Dir::Left);
								render = true;
								player_action
							},
							_ => PlayerAction::None,
						};

						match player_action {
							PlayerAction::KillCommonBeast(coord) => {
								self.state = GameState::Killing(Beat::One);
								if let Some(idx) = self.common_beasts.iter().position(|beast| beast.position == coord) {
									self.common_beasts.swap_remove(idx);
								}
							},
							PlayerAction::KillSuperBeast(coord) => {
								self.state = GameState::Killing(Beat::One);
								if let Some(idx) = self.super_beasts.iter().position(|beast| beast.position == coord) {
									self.super_beasts.swap_remove(idx);
								}
							},
							PlayerAction::KillEgg(coord) => {
								self.state = GameState::Killing(Beat::One);
								if let Some(idx) = self.eggs.iter().position(|egg| egg.position == coord) {
									self.eggs.swap_remove(idx);
								}
							},
							PlayerAction::KillHatchedBeast(coord) => {
								self.state = GameState::Killing(Beat::One);
								if let Some(idx) = self.hatched_beasts.iter().position(|beast| beast.position == coord) {
									self.hatched_beasts.swap_remove(idx);
								}
							},
							PlayerAction::KillPlayer => {
								self.state = GameState::Dying(Beat::One);
							},
							PlayerAction::None => {},
						}

						if render {
							// the player renders independent from the tick speed of the beasts
							self.render_with_state();
						}
					}
				} else {
					match byte as char {
						'q' | 'Q' => {
							self.state = GameState::Quit;
							break;
						},
						'h' | 'H' => {
							self.state = GameState::Help;
							break;
						},
						's' | 'S' => {
							self.state = GameState::HighScore;
							break;
						},
						_ => {},
					}
				}
			}

			// end game through time has ran out or no more lives
			if self.player.lives == 0 || self.get_secs_remaining() == 0 {
				self.state = GameState::GameOver;
				self.render_with_state();
				break;
			}

			// eggs hatching
			self.eggs.retain_mut(|egg| match egg.hatch(self.level.get_config()) {
				HatchingState::Incubating => true,
				HatchingState::Hatching(position, instant) => {
					self.board[position] = Tile::EggHatching(instant);
					true
				},
				HatchingState::Hatched(position) => {
					self.hatched_beasts.push(HatchedBeast::new(position));
					self.board[position] = Tile::HatchedBeast;
					false
				},
			});

			// end game through no more beasts
			if self.common_beasts.len() + self.super_beasts.len() + self.eggs.len() + self.hatched_beasts.len() == 0 {
				let secs_remaining = self.get_secs_remaining();
				self.player.score += secs_remaining as u16 / 10;

				if let Some(level) = self.level.next() {
					self.level = level;

					let board_terrain_info = Board::generate_terrain(level);
					self.board = Board::new(board_terrain_info.data);
					self.level = level;
					self.level_start = Instant::now();
					self.common_beasts = board_terrain_info.common_beasts;
					self.super_beasts = board_terrain_info.super_beasts;
					self.eggs = board_terrain_info.eggs;
					self.hatched_beasts = board_terrain_info.hatched_beasts;
					self.player.position = board_terrain_info.player.position;
					self.player.score += self.level.get_config().completion_score;
					last_tick = Instant::now() - TICK_DURATION;
				} else {
					self.state = GameState::Won;
					break;
				}
			}

			// game tick
			if last_tick.elapsed() >= TICK_DURATION {
				if matches!(self.beat, Beat::Five) {
					// beast movements
					for common_beasts in &mut self.common_beasts {
						if matches!(common_beasts.advance(&mut self.board, self.player.position), BeastAction::PlayerKilled) {
							self.player.lives -= 1;
							self.player.respawn(&mut self.board);
							self.state = GameState::Dying(Beat::One);
						}
					}
					for super_beasts in &mut self.super_beasts {
						if matches!(super_beasts.advance(&mut self.board, self.player.position), BeastAction::PlayerKilled) {
							self.player.lives -= 1;
							self.player.respawn(&mut self.board);
							self.state = GameState::Dying(Beat::One);
						}
					}
					for hatched_beasts in &mut self.hatched_beasts {
						if matches!(hatched_beasts.advance(&mut self.board, self.player.position), BeastAction::PlayerKilled) {
							self.player.lives -= 1;
							self.player.respawn(&mut self.board);
							self.state = GameState::Dying(Beat::One);
						}
					}
				}

				// end game through no more lives left
				if self.player.lives == 0 {
					self.state = GameState::GameOver;
					break;
				}

				// render with Dying and Killing animation
				self.render_with_state();
				self.beat = self.beat.next();
				last_tick = Instant::now();
			}
		}
	}

	fn handle_death_state(&mut self) {
		println!("{}", self.render_end_screen());

		loop {
			if let Ok(byte) = self.input_listener.try_recv() {
				match byte as char {
					' ' => {
						let board_terrain_info = Board::generate_terrain(Level::One);
						self.board = Board::new(board_terrain_info.data);
						self.level = Level::One;
						self.level_start = Instant::now();
						self.common_beasts = board_terrain_info.common_beasts;
						self.super_beasts = board_terrain_info.super_beasts;
						self.eggs = board_terrain_info.eggs;
						self.hatched_beasts = board_terrain_info.hatched_beasts;
						self.player = board_terrain_info.player;

						self.state = GameState::Playing;
						break;
					},
					'h' | 'H' => {
						self.state = GameState::Help;
						break;
					},
					's' | 'S' => {
						self.state = GameState::HighScore;
						break;
					},
					'q' | 'Q' => {
						self.state = GameState::Quit;
						break;
					},
					_ => {},
				}
			}
		}
	}

	fn handle_win_state(&mut self) {
		println!("{}", self.render_winning_screen());

		loop {
			if let Ok(byte) = self.input_listener.try_recv() {
				match byte as char {
					' ' => {
						let board_terrain_info = Board::generate_terrain(Level::One);
						self.board = Board::new(board_terrain_info.data);
						self.level = Level::One;
						self.level_start = Instant::now();
						self.common_beasts = board_terrain_info.common_beasts;
						self.super_beasts = board_terrain_info.super_beasts;
						self.eggs = board_terrain_info.eggs;
						self.hatched_beasts = board_terrain_info.hatched_beasts;
						self.player = board_terrain_info.player;

						self.state = GameState::Playing;
						break;
					},
					'h' | 'H' => {
						self.state = GameState::Help;
						break;
					},
					's' | 'S' => {
						self.state = GameState::HighScore;
						break;
					},
					'q' | 'Q' => {
						self.state = GameState::Quit;
						break;
					},
					_ => {},
				}
			}
		}
	}

	fn handle_help_state(&mut self) {
		let pause = Instant::now();
		let mut help = Help::new();
		println!("{}", help.render());

		loop {
			if let Ok(byte) = self.input_listener.try_recv() {
				if byte == 0x1B {
					let second = self.input_listener.recv().unwrap_or(0);
					let third = self.input_listener.recv().unwrap_or(0);
					if second == b'[' {
						let mut render = false;
						match third {
							b'C' => {
								help.next_page();
								render = true;
							},
							b'D' => {
								help.previous_page();
								render = true;
							},
							_ => {},
						}

						if render {
							println!("{}", help.render());
						}
					}
				} else {
					match byte as char {
						' ' => {
							self.level_start += pause.elapsed();
							self.state = GameState::Playing;
							break;
						},
						's' | 'S' => {
							self.state = GameState::HighScore;
							break;
						},
						'q' | 'Q' => {
							self.state = GameState::Quit;
							break;
						},
						_ => {},
					}
				}
			}
		}
	}

	fn handle_highscore_state(&mut self) {
		let pause = Instant::now();
		let mut highscore = Highscore::new();
		println!("{}", highscore.render());

		loop {
			if let Ok(byte) = self.input_listener.try_recv() {
				if byte == 0x1B {
					let second = self.input_listener.recv().unwrap_or(0);
					let third = self.input_listener.recv().unwrap_or(0);
					if second == b'[' {
						let mut render = false;
						match third {
							b'A' => {
								if *highscore.state.lock().unwrap() == State::Idle {
									highscore.scroll_up();
									render = true;
								}
							},
							b'B' => {
								if *highscore.state.lock().unwrap() == State::Idle {
									highscore.scroll_down();
									render = true;
								}
							},
							_ => {},
						}

						if render {
							println!("{}", highscore.render());
						}
					}
				} else {
					match byte as char {
						'r' | 'R' => {
							if let Ok(mut state) = highscore.state.lock() {
								if *state == State::Idle || *state == State::Error {
									*state = State::Loading;
									highscore.fetch_data();
									highscore.render_loading();
									println!("{}", Highscore::render_loading_screen());
								}
							}
						},
						' ' => {
							if let Ok(mut state) = highscore.state.lock() {
								*state = State::Quit;
							}
							self.level_start += pause.elapsed();
							self.state = GameState::Playing;
							break;
						},
						'h' | 'H' => {
							self.state = GameState::Help;
							break;
						},
						'q' | 'Q' => {
							self.state = GameState::Quit;
							break;
						},
						_ => {},
					}
				}
			}
		}
	}

	fn get_secs_remaining(&self) -> u64 {
		let elapsed = Instant::now().duration_since(self.level_start);
		let total_time = self.level.get_config().time;
		if total_time > elapsed {
			total_time - elapsed
		} else {
			Duration::from_secs(0)
		}
		.as_secs()
	}

	fn render_header(output: &mut String) {
		output.push('\n');
		output.push_str(" ╔╗  ╔═╗ ╔═╗ ╔═╗ ╔╦╗\n");
		output.push_str(" ╠╩╗ ║╣  ╠═╣ ╚═╗  ║\n");
		output.push_str(" ╚═╝ ╚═╝ ╩ ╩ ╚═╝  ╩\n");
	}

	fn render_footer(&self) -> String {
		let mut output = String::new();
		let secs_remaining = self.get_secs_remaining();
		let minutes = secs_remaining / 60;
		let seconds = secs_remaining % 60;
		let elapsed = self.level_start.elapsed();
		let tick_count = elapsed.as_millis() / TICK_DURATION.as_millis();
		let timer_color = if tick_count % 2 == 0 && minutes == 0 && seconds < 20 {
			"\x1b[31m"
		} else {
			"\x1b[39m"
		};

		output.push_str("⌂⌂                                        ");
		output.push_str("  Level: ");
		output.push_str(&format!("{ANSI_BOLD}{:0>2}{ANSI_RESET}", self.level.to_string()));
		output.push_str("  Beasts: ");
		output.push_str(&format!(
			"{ANSI_BOLD}{:0>2}{ANSI_RESET}",
			(self.common_beasts.len() + self.super_beasts.len() + self.hatched_beasts.len()).to_string()
		));
		output.push_str("  Lives: ");
		output.push_str(&format!("{ANSI_BOLD}{:0>2}{ANSI_RESET}", self.player.lives.to_string()));
		output.push_str("  Time: ");
		output.push_str(&format!("{ANSI_BOLD}{timer_color}{:02}:{:02}{ANSI_RESET}", minutes, seconds));
		output.push_str("  Score: ");
		output.push_str(&format!("{ANSI_BOLD}{:0>4}{ANSI_RESET}", self.player.score));
		output.push_str("\n\n");

		output
	}

	fn render_top_frame() -> String {
		format!("\x1b[33m▛{}▜ \x1b[39m\n", "▀▀".repeat(BOARD_WIDTH))
	}

	fn render_bottom_frame() -> String {
		format!("\x1b[33m▙{}▟  \x1b[39m\n", "▄▄".repeat(BOARD_WIDTH))
	}

	fn render_intro() -> String {
		let mut output = String::new();
		Self::render_header(&mut output);
		output.push_str(&Self::render_top_frame());
		output.push_str(&LOGO.join("\n"));
		output.push('\n');
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                               Written and Developed by the following                               \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                         Dominik Wilkowski                                          \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                               Faithfully recreated from the work of                                \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                      Dan Baker , Alan Brown , Mark Hamilton , Derrick Shadel                       \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m             NOTICE:    This is a Free copy of BEAST. You may copy it and give it away.             \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                        If you enjoy the game, please send a contribution ($20) to                  \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                        Dan Baker, PO BOX 1174, Orem UT 84057                                       \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m                                     Press {ANSI_BOLD}[SPACE]{ANSI_RESET} key to start                                     \x1b[33m▐\x1b[39m\n"));
		output.push_str(&format!("\x1b[33m▌\x1b[39m                                 {ANSI_BOLD}[Q]{ANSI_RESET} Quit  {ANSI_BOLD}[H]{ANSI_RESET} Help  {ANSI_BOLD}[S]{ANSI_RESET} Highscores                                 \x1b[33m▐\x1b[39m\n"));
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&Self::render_bottom_frame());
		output.push_str("\n\n");

		output
	}

	fn render_end_screen(&self) -> String {
		let mut output = String::new();
		let top_pos = format!("\x1b[{}F", ANSI_FRAME_SIZE + ANSI_BOARD_HEIGHT + ANSI_FRAME_SIZE + ANSI_FOOTER_HEIGHT);

		output.push_str(&top_pos);
		output.push_str(&LOGO.join("\n"));
		output.push('\n');
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		if self.player.lives == 0 {
			output.push_str(&format!("\x1b[33m▌\x1b[39m                                              {ANSI_BOLD}YOU DIED{ANSI_RESET}                                              \x1b[33m▐\x1b[39m\n"));
		} else {
			output.push_str(&format!("\x1b[33m▌\x1b[39m                                          {ANSI_BOLD}YOUR TIME RAN OUT{ANSI_RESET}                                         \x1b[33m▐\x1b[39m\n"));
		}
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m     {ANSI_BOLD}SCORE{ANSI_RESET}: {:0>4}                                                                                    \x1b[33m▐\x1b[39m\n", self.player.score));
		output.push_str(&format!("\x1b[33m▌\x1b[39m     {ANSI_BOLD}BEASTS KILLED{ANSI_RESET}: {:<2}                                                                              \x1b[33m▐\x1b[39m\n", self.player.beasts_killed.to_string()));
		output.push_str(&format!("\x1b[33m▌\x1b[39m     {ANSI_BOLD}LEVEL REACHED{ANSI_RESET}: {:<2}                                                                              \x1b[33m▐\x1b[39m\n", self.level.to_string()));
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m                                  Press {ANSI_BOLD}[SPACE]{ANSI_RESET} key to play again                                   \x1b[33m▐\x1b[39m\n"));
		output.push_str(&format!("\x1b[33m▌\x1b[39m                                     Press {ANSI_BOLD}[Q]{ANSI_RESET} to exit the game                                     \x1b[33m▐\x1b[39m\n"));
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&Self::render_bottom_frame());
		output.push_str("\n\n");

		output
	}

	fn render_winning_screen(&self) -> String {
		let mut output = String::new();
		let top_pos = format!("\x1b[{}F", ANSI_FRAME_SIZE + ANSI_BOARD_HEIGHT + ANSI_FRAME_SIZE + ANSI_FOOTER_HEIGHT);

		output.push_str(&top_pos);
		output.push_str(&LOGO.join("\n"));
		output.push('\n');
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m                                               {ANSI_BOLD}YOU WON{ANSI_RESET}                                              \x1b[33m▐\x1b[39m\n"));
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m     {ANSI_BOLD}SCORE{ANSI_RESET}: {:0>4}                                                                                    \x1b[33m▐\x1b[39m\n", self.player.score));
		output.push_str(&format!("\x1b[33m▌\x1b[39m     {ANSI_BOLD}BEASTS KILLED{ANSI_RESET}: {:<2}                                                                              \x1b[33m▐\x1b[39m\n", self.player.beasts_killed.to_string()));
		output.push_str(&format!("\x1b[33m▌\x1b[39m     {ANSI_BOLD}LEVEL REACHED{ANSI_RESET}: {:<2}                                                                              \x1b[33m▐\x1b[39m\n", self.level.to_string()));
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m                                  Press {ANSI_BOLD}[SPACE]{ANSI_RESET} key to play again                                   \x1b[33m▐\x1b[39m\n"));
		output.push_str(&format!("\x1b[33m▌\x1b[39m                                     Press {ANSI_BOLD}[Q]{ANSI_RESET} to exit the game                                     \x1b[33m▐\x1b[39m\n"));
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&Self::render_bottom_frame());
		output.push_str("\n\n");

		output
	}

	fn render_board(&self) -> String {
		let top_pos = format!("\x1b[{}F", ANSI_FRAME_SIZE + ANSI_BOARD_HEIGHT + ANSI_FRAME_SIZE + ANSI_FOOTER_HEIGHT);
		let bottom_pos = format!("\x1b[{}E", ANSI_FRAME_SIZE);
		let mut output = String::new();

		output.push_str(&top_pos);
		output.push_str(&self.board.render());
		output.push_str(&Self::render_bottom_frame());
		output.push_str(&self.render_footer());
		output.push_str(&bottom_pos);
		output
	}

	fn render_with_state(&mut self) {
		match self.state {
			GameState::Dying(beat) => match beat {
				Beat::One => {
					self.state = GameState::Dying(Beat::Two);
					print!("\x1b[48;5;196m");
				},
				Beat::Two => {
					self.state = GameState::Dying(Beat::Three);
					print!("\x1b[48;5;208m");
				},
				Beat::Three | Beat::Four | Beat::Five => {
					self.state = GameState::Playing;
					print!("\x1b[49m");
				},
			},
			GameState::Killing(beat) => match beat {
				Beat::One => {
					self.state = GameState::Killing(Beat::Two);
					print!("\x1b[48;2;51;51;51m");
				},
				Beat::Two | Beat::Three | Beat::Four | Beat::Five => {
					self.state = GameState::Playing;
					print!("\x1b[49m");
				},
			},
			GameState::Playing => {
				print!("\x1b[49m");
			},
			_ => {},
		}
		print!("{}", self.render_board());
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::{BOARD_WIDTH, common::strip_ansi_border};

	pub const ANSI_HEADER_HEIGHT: usize = 4;

	#[test]
	fn beat_next_test() {
		assert_eq!(Beat::One.next(), Beat::Two, "Beat should go from One to Two");
		assert_eq!(Beat::Two.next(), Beat::Three, "Beat should go from Two to Three");
		assert_eq!(Beat::Three.next(), Beat::Four, "Beat should go from Three to Four");
		assert_eq!(Beat::Four.next(), Beat::Five, "Beat should go from Four to Five");
		assert_eq!(Beat::Five.next(), Beat::One, "Beat should go from Five to One");
	}

	#[test]
	fn game_new_test() {
		let game = Game::new();

		assert_eq!(game.state, GameState::Intro, "Game should start in Intro state");
		assert_eq!(game.beat, Beat::One, "Game should start with Beat One");
		assert_eq!(game.level, Level::One, "Game should start with Level One");

		for common_beast in &game.common_beasts {
			assert_eq!(
				game.board[common_beast.position],
				Tile::CommonBeast,
				"Each common beast is placed on a the board with a CommonBeast tile"
			);
		}
		for super_beast in &game.super_beasts {
			assert_eq!(
				game.board[super_beast.position],
				Tile::SuperBeast,
				"Each super beast is placed on a the board with a SuperBeast tile"
			);
		}
		for egg in &game.eggs {
			assert!(matches!(game.board[egg.position], Tile::Egg(_)), "Each egg is placed on a the board with an Egg tile");
		}
		for hatched_beast in &game.hatched_beasts {
			assert_eq!(
				game.board[hatched_beast.position],
				Tile::HatchedBeast,
				"Each hatched beast is placed on a the board with a HatchedBeast tile"
			);
		}
		assert_eq!(
			game.board[game.player.position],
			Tile::Player,
			"Each player is placed on a the board with a Player tile"
		);

		assert_eq!(game.player.lives, 5, "Each player should start with 5 lives");
		assert_eq!(game.player.score, 0, "Each player should start with a score of 0");
	}

	#[test]
	fn get_secs_remaining_test() {
		let mut game = Game::new();

		let now = Instant::now();
		game.level_start = now - Duration::from_secs(10);

		let expected_remaining = game.level.get_config().time.as_secs() - 11;
		assert_eq!(game.get_secs_remaining(), expected_remaining, "Calculate the remaining time");

		game.level_start = now - game.level.get_config().time - Duration::from_secs(5);
		assert_eq!(game.get_secs_remaining(), 0, "Calculate the remaining time when more time has passed than we expect");
	}

	#[test]
	fn render_footer_test() {
		let game = Game::new();
		let footer = game.render_footer();

		assert!(footer.contains("Level:"), "Footer should contain Level");
		assert!(footer.contains("Beasts:"), "Footer should contain Beasts");
		assert!(footer.contains("Lives:"), "Footer should contain Lives");
		assert!(footer.contains("Time:"), "Footer should contain Time");
		assert!(footer.contains("Score:"), "Footer should contain Score");
	}

	#[test]
	fn render_with_state_test() {
		let mut game = Game::new();

		game.state = GameState::Intro;
		game.render_with_state();
		assert_eq!(game.state, GameState::Intro, "The intro state should remain the same");

		game.state = GameState::Playing;
		game.render_with_state();
		assert_eq!(game.state, GameState::Playing, "The playing state should remain the same");

		game.state = GameState::Help;
		game.render_with_state();
		assert_eq!(game.state, GameState::Help, "The help state should remain the same");

		game.state = GameState::HighScore;
		game.render_with_state();
		assert_eq!(game.state, GameState::HighScore, "The highscore state should remain the same");

		game.state = GameState::GameOver;
		game.render_with_state();
		assert_eq!(game.state, GameState::GameOver, "The gameover state should remain the same");

		game.state = GameState::Won;
		game.render_with_state();
		assert_eq!(game.state, GameState::Won, "The won state should remain the same");

		game.state = GameState::Quit;
		game.render_with_state();
		assert_eq!(game.state, GameState::Quit, "The quit state should remain the same");

		game.state = GameState::Dying(Beat::One);
		game.render_with_state();
		assert_eq!(game.state, GameState::Dying(Beat::Two), "The dying state moves to the second beat");
		game.render_with_state();
		assert_eq!(game.state, GameState::Dying(Beat::Three), "The dying state moves to the third beat");
		game.render_with_state();
		assert_eq!(game.state, GameState::Playing, "The dying state moves to the playing state");

		game.state = GameState::Killing(Beat::One);
		game.render_with_state();
		assert_eq!(game.state, GameState::Killing(Beat::Two), "The killing state moves to the second beat");
		game.render_with_state();
		assert_eq!(game.state, GameState::Playing, "The killing state moves to the playing state");
	}

	#[test]
	fn header_height_test() {
		let mut output = String::new();
		Game::render_header(&mut output);
		assert_eq!(
			output.lines().count(),
			ANSI_HEADER_HEIGHT,
			"There should be exactly ANSI_HEADER_HEIGHT lines in the header"
		);
	}

	#[test]
	fn footer_height_test() {
		assert_eq!(
			Game::new().render_footer().lines().count(),
			ANSI_FOOTER_HEIGHT,
			"There should be exactly ANSI_FOOTER_HEIGHT lines in the footer"
		);
	}

	#[test]
	fn footer_line_length_test() {
		let output = Game::new().render_footer();

		let lines = output.lines().collect::<Vec<&str>>();
		for (i, line) in lines.iter().enumerate() {
			if i < lines.len() - 1 {
				assert_eq!(
					strip_ansi_border(line).len(),
					BOARD_WIDTH * 2 + ANSI_FRAME_SIZE + ANSI_FRAME_SIZE,
					"Line {i} should be the correct length"
				);
			}
		}
	}

	#[test]
	fn top_frame_height_test() {
		assert_eq!(
			Game::render_top_frame().lines().count(),
			ANSI_FRAME_SIZE,
			"There should be exactly ANSI_FRAME_HEIGHT lines in the top frame"
		);
	}

	#[test]
	fn top_frame_line_length_test() {
		let output = Game::render_top_frame();

		let lines = output.lines().collect::<Vec<&str>>();
		for (i, line) in lines.iter().enumerate() {
			if i < lines.len() - 1 {
				assert_eq!(
					strip_ansi_border(line).len(),
					BOARD_WIDTH * 2 + ANSI_FRAME_SIZE + ANSI_FRAME_SIZE,
					"Line {i} should be the correct length"
				);
			}
		}
	}

	#[test]
	fn bottom_frame_height_test() {
		assert_eq!(
			Game::render_bottom_frame().lines().count(),
			ANSI_FRAME_SIZE,
			"There should be exactly ANSI_FRAME_HEIGHT lines in the bottom frame"
		);
	}

	#[test]
	fn bottom_frame_line_length_test() {
		let output = Game::render_bottom_frame();

		let lines = output.lines().collect::<Vec<&str>>();
		for (i, line) in lines.iter().enumerate() {
			if i < lines.len() - 1 {
				assert_eq!(
					strip_ansi_border(line).len(),
					BOARD_WIDTH * 2 + ANSI_FRAME_SIZE + ANSI_FRAME_SIZE,
					"Line {i} should be the correct length"
				);
			}
		}
	}

	#[test]
	fn intro_height_test() {
		assert_eq!(
			Game::render_intro().lines().count(),
			ANSI_HEADER_HEIGHT + ANSI_FRAME_SIZE + ANSI_BOARD_HEIGHT + ANSI_FRAME_SIZE + ANSI_FOOTER_HEIGHT,
			"The intro screen needs to be the correct height for the ANSI re-render to work"
		);
	}

	#[test]
	fn intro_line_length_test() {
		let output = Game::render_intro();

		let lines = output.lines().skip(5).collect::<Vec<&str>>();
		for (i, line) in lines.iter().enumerate() {
			if i < lines.len() - 3 {
				assert_eq!(
					strip_ansi_border(line).len(),
					BOARD_WIDTH * 2,
					"Line {i} should be the correct length is={:?}",
					strip_ansi_border(line)
				);
			}
		}
	}

	#[test]
	fn end_screen_height_test() {
		assert_eq!(
			Game::new().render_end_screen().lines().count(),
			ANSI_BOARD_HEIGHT + ANSI_FRAME_SIZE + ANSI_FOOTER_HEIGHT,
			"The end screen needs to be the correct height for the ANSI re-render to work"
		);
	}

	#[test]
	fn end_screen_line_length_test() {
		let output = Game::new().render_end_screen();

		let lines = output.lines().collect::<Vec<&str>>();
		for (i, line) in lines.iter().enumerate() {
			if i < lines.len() - 3 {
				assert_eq!(
					strip_ansi_border(line).len(),
					BOARD_WIDTH * 2,
					"Line {i} should be the correct length is={:?}",
					strip_ansi_border(line)
				);
			}
		}
	}

	#[test]
	fn winning_screen_height_test() {
		assert_eq!(
			Game::new().render_winning_screen().lines().count(),
			ANSI_BOARD_HEIGHT + ANSI_FRAME_SIZE + ANSI_FOOTER_HEIGHT,
			"The winning screen needs to be the correct height for the ANSI re-render to work"
		);
	}

	#[test]
	fn winning_screen_line_length_test() {
		let output = Game::new().render_winning_screen();

		let lines = output.lines().collect::<Vec<&str>>();
		for (i, line) in lines.iter().enumerate() {
			if i < lines.len() - 3 {
				assert_eq!(
					strip_ansi_border(line).len(),
					BOARD_WIDTH * 2,
					"Line {i} should be the correct length is={:?}",
					strip_ansi_border(line)
				);
			}
		}
	}

	#[test]
	fn render_end_screen_message_test() {
		let mut game = Game::new();

		let end_screen = game.render_end_screen();
		assert!(end_screen.contains("YOUR TIME RAN OUT"), "End screen should say 'YOUR TIME RAN OUT' when lives > 0");

		game.player.lives = 0;
		let end_screen = game.render_end_screen();
		assert!(end_screen.contains("YOU DIED"), "End screen should say 'YOU DIED' when lives == 0");
	}

	#[test]
	fn render_footer_time_format_test() {
		let mut game = Game::new();

		let test_times = [(0, "00:00"), (10, "00:09"), (60, "00:59"), (75, "01:14")];

		for (secs, expected) in test_times {
			game.level_start = Instant::now() - (game.level.get_config().time - Duration::from_secs(secs));

			let footer = game.render_footer();

			assert!(footer.contains(expected), "Footer should display '{expected}' when {secs} seconds remain");
		}
	}

	#[test]
	fn play_quit_test() {
		let mut game = Game::new();
		let (sender, receiver) = mpsc::channel::<u8>();
		game.input_listener = receiver;

		let handle = thread::spawn(move || {
			game.play();
			game
		});

		sender.send(b'q').unwrap();
		thread::sleep(Duration::from_millis(50));
		let game = handle.join().unwrap();
		assert_eq!(game.state, GameState::Quit, "The game has quit after you hit the 'q' key");
	}
}
