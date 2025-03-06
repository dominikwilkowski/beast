use std::{
	io::{self, Read},
	sync::mpsc,
	thread,
	time::{Duration, Instant},
};

use crate::{
	BOARD_HEIGHT, BOARD_WIDTH, Dir, Tile,
	beasts::{CommonBeast, Egg, HatchedBeast, SuperBeast},
	board::Board,
	levels::Level,
	player::{Player, PlayerKill},
	raw_mode::{RawMode, install_raw_mode_signal_handler},
};

pub const ANSI_BOARD_HEIGHT: usize = BOARD_HEIGHT;
pub const ANSI_FRAME_HEIGHT: usize = 1;
pub const ANSI_FOOTER_HEIGHT: usize = 2;
const ANSI_BOLD: &str = "\x1B[1m";
const ANSI_RESET: &str = "\x1B[0m";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
	Intro,
	Playing,
	Help,
	Settings,
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
	pub input_listener: mpsc::Receiver<u8>,
	pub _raw_mode: RawMode,
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
				GameState::Playing => {
					self.handle_playing_state(last_tick);
				},
				GameState::Help => {
					self.handle_help_state();
				},
				GameState::Settings => {
					self.handle_settings_state();
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
						self.state = GameState::Settings;
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
		print!("{}", self.re_render());

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
							_ => PlayerKill::None,
						};

						match player_action {
							PlayerKill::KillCommonBeast(coord) => {
								if let Some(idx) = self.common_beasts.iter().position(|beast| beast.position == coord) {
									self.common_beasts.swap_remove(idx);
								}
							},
							PlayerKill::KillSuperBeast(coord) => {
								if let Some(idx) = self.super_beasts.iter().position(|beast| beast.position == coord) {
									self.super_beasts.swap_remove(idx);
								}
							},
							PlayerKill::KillEgg(coord) => {
								if let Some(idx) = self.eggs.iter().position(|egg| egg.position == coord) {
									self.eggs.swap_remove(idx);
								}
							},
							PlayerKill::KillHatchedBeast(coord) => {
								if let Some(idx) = self.hatched_beasts.iter().position(|beast| beast.position == coord) {
									self.hatched_beasts.swap_remove(idx);
								}
							},
							PlayerKill::None => {},
						}

						if render {
							print!("{}", self.re_render());
							last_tick = Instant::now();
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
							self.state = GameState::Settings;
							break;
						},
						_ => {},
					}
				}
			}

			let elapsed_time = Instant::now().duration_since(self.level_start);
			let total_time = self.level.get_config().time;

			if self.player.lives == 0 || elapsed_time >= total_time {
				self.state = GameState::GameOver;
				break;
			}

			if self.common_beasts.len() + self.super_beasts.len() + self.eggs.len() + self.hatched_beasts.len() == 0 {
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
					print!("{}", self.re_render());
					last_tick = Instant::now();
				} else {
					self.state = GameState::Won;
					break;
				}
			}

			if last_tick.elapsed() >= Duration::from_secs(1) {
				print!("{}", self.re_render());
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
						self.state = GameState::Settings;
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
						self.state = GameState::Settings;
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
		println!("{}", Self::render_help());

		loop {
			if let Ok(byte) = self.input_listener.try_recv() {
				match byte as char {
					' ' => {
						let pause_duration = pause.elapsed();
						self.level_start += pause_duration;
						self.state = GameState::Playing;
						break;
					},
					's' | 'S' => {
						self.state = GameState::Settings;
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

	// TODO: write a settings struct that can be rendered and owns the position of the cursor
	fn handle_settings_state(&mut self) {
		println!("Settings");

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

	fn get_remaining_time(&self) -> String {
		let elapsed = Instant::now().duration_since(self.level_start);
		let total_time = self.level.get_config().time;
		let time_remaining = if total_time > elapsed {
			total_time - elapsed
		} else {
			Duration::from_secs(0)
		}
		.as_secs();

		let minutes = time_remaining / 60;
		let seconds = time_remaining % 60;
		format!("{:02}:{:02}", minutes, seconds)
	}

	fn render_footer(&self) -> String {
		let mut output = String::new();

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
		output.push_str(&format!("{}{}{}", ANSI_BOLD, self.get_remaining_time(), ANSI_RESET));
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

	pub fn render_intro() -> String {
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

	pub fn render_help() -> String {
		let mut output = String::new();
		let top_pos = format!("\x1b[{}F", ANSI_FRAME_HEIGHT + ANSI_BOARD_HEIGHT + ANSI_FRAME_HEIGHT + ANSI_FOOTER_HEIGHT);

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
		output.push_str(&format!("\x1b[33m▌\x1b[39m                                                {ANSI_BOLD}HELP{ANSI_RESET}                                                \x1b[33m▐\x1b[39m\n"));
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m   You must survive while {ANSI_BOLD}beasts{ANSI_RESET} attack you. The only way to fight back is to squish the beasts     \x1b[33m▐\x1b[39m\n"));
		output.push_str("\x1b[33m▌\x1b[39m   Between blocks. But there are different types of beasts the longer you survive.                  \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m   You are {} and you move around with the arrow keys on your keyboard.                             \x1b[33m▐\x1b[39m\n", Tile::Player));
		output.push_str(&format!("\x1b[33m▌\x1b[39m   You can move {} around until you hit a {} which can't be moved.                                  \x1b[33m▐\x1b[39m\n", Tile::Block, Tile::StaticBlock));
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m   The {} is the common beast and can be squished against any blocks or board frame.                \x1b[33m▐\x1b[39m\n", Tile::CommonBeast));
		output.push_str(&format!("\x1b[33m▌\x1b[39m   In later levels you will encounter the {} super beast which can only be squished against a {}.   \x1b[33m▐\x1b[39m\n", Tile::SuperBeast, Tile::StaticBlock));
		output.push_str(&format!("\x1b[33m▌\x1b[39m   At the end you will encounter {} eggs which hatch into {} hatched beasts.                        \x1b[33m▐\x1b[39m\n", Tile::Egg, Tile::HatchedBeast));
		output.push_str(&format!("\x1b[33m▌\x1b[39m   These hatched beasts can be squished like {} common beasts but they can move {} and will try     \x1b[33m▐\x1b[39m\n", Tile::CommonBeast, Tile::Block));
		output.push_str("\x1b[33m▌\x1b[39m   to squish YOU!                                                                                   \x1b[33m▐\x1b[39m\n");
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m   And you better hurry up because you only a little time to survive the {ANSI_BOLD}BEAST ATTACK{ANSI_RESET}.              \x1b[33m▐\x1b[39m\n"));
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&format!("\x1b[33m▌\x1b[39m                               Press {ANSI_BOLD}[SPACE]{ANSI_RESET} key to get back to game                                \x1b[33m▐\x1b[39m\n"));
		output.push_str(&format!("\x1b[33m▌\x1b[39m                                     Press {ANSI_BOLD}[Q]{ANSI_RESET} to exit the game                                     \x1b[33m▐\x1b[39m\n"));
		output.push_str("\x1b[33m▌\x1b[39m                                                                                                    \x1b[33m▐\x1b[39m\n");
		output.push_str(&Self::render_bottom_frame());
		output.push_str("\n\n");

		output
	}

	pub fn render_end_screen(&self) -> String {
		let mut output = String::new();
		let top_pos = format!("\x1b[{}F", ANSI_FRAME_HEIGHT + ANSI_BOARD_HEIGHT + ANSI_FRAME_HEIGHT + ANSI_FOOTER_HEIGHT);

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

	pub fn render_winning_screen(&self) -> String {
		let mut output = String::new();
		let top_pos = format!("\x1b[{}F", ANSI_FRAME_HEIGHT + ANSI_BOARD_HEIGHT + ANSI_FRAME_HEIGHT + ANSI_FOOTER_HEIGHT);

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

	pub fn re_render(&self) -> String {
		let top_pos = format!("\x1b[{}F", ANSI_FRAME_HEIGHT + ANSI_BOARD_HEIGHT + ANSI_FRAME_HEIGHT + ANSI_FOOTER_HEIGHT);
		let bottom_pos = format!("\x1b[{}E", ANSI_FRAME_HEIGHT);
		let mut output = String::new();

		output.push_str(&top_pos);
		output.push_str(&self.board.render());
		output.push_str(&Self::render_bottom_frame());
		output.push_str(&self.render_footer());
		output.push_str(&bottom_pos);
		output
	}
}

#[cfg(test)]
mod test {
	use super::*;
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
}
