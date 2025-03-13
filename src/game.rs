//! this module contains the main struct that orchestrates the game

use std::{
	io::{self, Read},
	sync::mpsc,
	thread,
	time::{Duration, Instant},
};

use crate::{
	BOARD_HEIGHT, BOARD_WIDTH, Dir, Tile,
	beasts::{Beast, BeastAction, CommonBeast, Egg, HatchedBeast, HatchingState, SuperBeast},
	board::Board,
	help::Help,
	levels::Level,
	player::{Player, PlayerAction},
	raw_mode::{RawMode, install_raw_mode_signal_handler},
};

pub const ANSI_BOARD_HEIGHT: usize = BOARD_HEIGHT;
pub const ANSI_FRAME_SIZE: usize = 1;
pub const ANSI_FOOTER_HEIGHT: usize = 2;
pub const ANSI_BOLD: &str = "\x1B[1m";
pub const ANSI_RESET: &str = "\x1B[0m";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Frames {
	One,
	Two,
	Three,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
	Intro,
	Playing,
	Dying(Frames),
	Killing(Frames),
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
	input_listener: mpsc::Receiver<u8>,
	_raw_mode: RawMode,
}

impl Game {
	pub fn new() -> Self {
		let board_terrain_info = Board::generate_terrain(Level::One);

		install_raw_mode_signal_handler();
		let _raw_mode = RawMode::enter().unwrap_or_else(|error| {
			eprintln!("Raw mode could not be entered in this shell: {}", error);
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
					self.handle_death();
				},
				GameState::Won => {
					self.handle_win();
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

		let mut tick_count: u8 = 0;
		const TICK_DURATION: Duration = Duration::from_millis(200);
		const BEAST_MOVEMENT_PER_TICK: u8 = 5;

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
								self.state = GameState::Killing(Frames::One);
								if let Some(idx) = self.common_beasts.iter().position(|beast| beast.position == coord) {
									self.common_beasts.swap_remove(idx);
								}
							},
							PlayerAction::KillSuperBeast(coord) => {
								self.state = GameState::Killing(Frames::One);
								if let Some(idx) = self.super_beasts.iter().position(|beast| beast.position == coord) {
									self.super_beasts.swap_remove(idx);
								}
							},
							PlayerAction::KillEgg(coord) => {
								self.state = GameState::Killing(Frames::One);
								if let Some(idx) = self.eggs.iter().position(|egg| egg.position == coord) {
									self.eggs.swap_remove(idx);
								}
							},
							PlayerAction::KillHatchedBeast(coord) => {
								self.state = GameState::Killing(Frames::One);
								if let Some(idx) = self.hatched_beasts.iter().position(|beast| beast.position == coord) {
									self.hatched_beasts.swap_remove(idx);
								}
							},
							PlayerAction::KillPlayer => {
								self.state = GameState::Dying(Frames::One);
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
				break;
			}

			// eggs hatching
			self.eggs.retain(|egg| match egg.hatch(self.level.get_config()) {
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
				tick_count += 1;

				if tick_count == BEAST_MOVEMENT_PER_TICK {
					tick_count = 0;

					// beast movements
					for common_beasts in &mut self.common_beasts {
						if matches!(common_beasts.advance(&mut self.board, self.player.position), BeastAction::PlayerKilled) {
							self.player.lives -= 1;
							self.player.respawn(&mut self.board);
							self.state = GameState::Dying(Frames::One);
						}
					}
					for super_beasts in &mut self.super_beasts {
						if matches!(super_beasts.advance(&mut self.board, self.player.position), BeastAction::PlayerKilled) {
							self.player.lives -= 1;
							self.player.respawn(&mut self.board);
							self.state = GameState::Dying(Frames::One);
						}
					}
					for hatched_beasts in &mut self.hatched_beasts {
						if matches!(hatched_beasts.advance(&mut self.board, self.player.position), BeastAction::PlayerKilled) {
							self.player.lives -= 1;
							self.player.respawn(&mut self.board);
							self.state = GameState::Dying(Frames::One);
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
				last_tick = Instant::now();
			}
		}
	}

	fn handle_death(&mut self) {
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

	fn handle_win(&mut self) {
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

	// TODO: write a highscore struct that can be rendered and owns the position of the cursor
	fn handle_highscore_state(&mut self) {
		println!("HighScore");

		loop {
			if let Ok(byte) = self.input_listener.try_recv() {
				match byte as char {
					' ' => {
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

	fn render_header(output: &mut String) {
		output.push('\n');
		output.push_str(" ╔╗  ╔═╗ ╔═╗ ╔═╗ ╔╦╗\n");
		output.push_str(" ╠╩╗ ║╣  ╠═╣ ╚═╗  ║\n");
		output.push_str(" ╚═╝ ╚═╝ ╩ ╩ ╚═╝  ╩\n");
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

	fn render_footer(&self) -> String {
		let mut output = String::new();
		let secs_remaining = self.get_secs_remaining();

		output.push_str("⌂⌂                                        ");
		output.push_str("  Level: ");
		output.push_str(&format!("{}{:0>2}{}", ANSI_BOLD, self.level.to_string(), ANSI_RESET));
		output.push_str("  Beasts: ");
		output.push_str(&format!(
			"{}{:0>2}{}",
			ANSI_BOLD,
			(self.common_beasts.len() + self.super_beasts.len() + self.hatched_beasts.len()).to_string(),
			ANSI_RESET
		));
		output.push_str("  Lives: ");
		output.push_str(&format!("{}{:0>2}{}", ANSI_BOLD, self.player.lives.to_string(), ANSI_RESET));
		output.push_str("  Time: ");
		output.push_str(&format!("{}{:02}:{:02}{}", ANSI_BOLD, secs_remaining / 60, secs_remaining % 60, ANSI_RESET));
		output.push_str("  Score: ");
		output.push_str(&format!("{}{:0>4}{}", ANSI_BOLD, self.player.score, ANSI_RESET));
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
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                 HHHH    HHHHH   HHH    HHHH  HHHHH                                 \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                 H   H   H      H   H  H        H                                   \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                 H   H   H      H   H  H        H                                   \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                 HHHH    HHHH   HHHHH   HHH     H                                   \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                 H   H   H      H   H      H    H                                   \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                 H   H   H      H   H      H    H                                   \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                 HHHH    HHHHH  H   H  HHHH     H                                   \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
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
		output.push_str(&format!("\x1b[33m▌\x1b[39m                             Press {ANSI_BOLD}[H]{ANSI_RESET} for help or {ANSI_BOLD}[Q]{ANSI_RESET} to exit the game                             \x1b[33m▐\x1b[39m\n"));
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&Self::render_bottom_frame());
		output.push_str("\n\n");

		output
	}

	fn render_end_screen(&self) -> String {
		let mut output = String::new();
		let top_pos = format!("\x1b[{}F", ANSI_FRAME_SIZE + ANSI_BOARD_HEIGHT + ANSI_FRAME_SIZE + ANSI_FOOTER_HEIGHT);

		output.push_str(&top_pos);
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                 HHHH    HHHHH   HHH    HHHH  HHHHH                                 \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                 H   H   H      H   H  H        H                                   \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                 H   H   H      H   H  H        H                                   \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                 HHHH    HHHH   HHHHH   HHH     H                                   \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                 H   H   H      H   H      H    H                                   \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                 H   H   H      H   H      H    H                                   \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                 HHHH    HHHHH  H   H  HHHH     H                                   \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
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
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                 HHHH    HHHHH   HHH    HHHH  HHHHH                                 \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                 H   H   H      H   H  H        H                                   \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                 H   H   H      H   H  H        H                                   \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                 HHHH    HHHH   HHHHH   HHH     H                                   \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                 H   H   H      H   H      H    H                                   \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                 H   H   H      H   H      H    H                                   \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                 HHHH    HHHHH  H   H  HHHH     H                                   \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
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
			GameState::Dying(frame) => match frame {
				Frames::One => {
					self.state = GameState::Dying(Frames::Two);
					print!("\x1b[48;5;196m");
				},
				Frames::Two => {
					self.state = GameState::Dying(Frames::Three);
					print!("\x1b[48;5;208m");
				},
				Frames::Three => {
					self.state = GameState::Playing;
					print!("\x1b[49m");
				},
			},
			GameState::Killing(frame) => match frame {
				Frames::One => {
					self.state = GameState::Killing(Frames::Two);
					print!("\x1b[48;2;51;51;51m");
				},
				Frames::Two | Frames::Three => {
					self.state = GameState::Killing(Frames::Three);
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
					"Line {} should be the correct length",
					i
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
					"Line {} should be the correct length",
					i
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
					"Line {} should be the correct length",
					i
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
					"Line {} should be the correct length is={:?}",
					i,
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
					"Line {} should be the correct length is={:?}",
					i,
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
					"Line {} should be the correct length is={:?}",
					i,
					strip_ansi_border(line)
				);
			}
		}
	}
}
