use std::{
	// time::{Duration, Instant},
	fmt::Write,
	io::{self, Read},
	sync::mpsc,
	thread,
};

use crate::{
	BOARD_HEIGHT, BOARD_WIDTH, Level,
	board::Board,
	movement::{Dir, move_player},
	raw_mode::{RawMode, install_raw_mode_signal_handler},
};

pub const ANSI_BOARD_HEIGHT: usize = BOARD_HEIGHT;
pub const ANSI_FRAME_HEIGHT: usize = 1;
pub const ANSI_FOOTER_HEIGHT: usize = 2;

pub struct Game {
	pub board: Board,
}

impl Game {
	pub fn new() -> Self {
		Self {
			board: Board::new(Level::One),
		}
	}

	pub fn input_listener(&mut self) -> io::Result<()> {
		// let mut last_tick = Instant::now();

		install_raw_mode_signal_handler();
		let _raw_mode = RawMode::enter()?;
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

		loop {
			if let Ok(byte) = receiver.try_recv() {
				if byte == 0x1B {
					let second = receiver.recv().unwrap_or(0);
					let third = receiver.recv().unwrap_or(0);
					if second == b'[' {
						match third {
							b'A' => {
								move_player(&mut self.board, &Dir::Up);
								print!("{}", self.re_render());
							},
							b'C' => {
								move_player(&mut self.board, &Dir::Right);
								print!("{}", self.re_render());
							},
							b'B' => {
								move_player(&mut self.board, &Dir::Down);
								print!("{}", self.re_render());
							},
							b'D' => {
								move_player(&mut self.board, &Dir::Left);
								print!("{}", self.re_render());
							},
							_ => {},
						}
					}
				} else {
					match byte as char {
						'q' => {
							println!("Bye...");
							break;
						},
						// TODO: support other keys
						_ => {},
					}
				}
			}

			// if last_tick.elapsed() >= Duration::from_millis(16) {
			// 	// print!("{reset_pos}{}", self.render::<(), &str>(None));
			// 	last_tick = Instant::now();
			// }
		}

		Ok(())
	}

	pub fn play() {}

	fn render_header(&self, output: &mut String) {
		output.push('\n');
		output.push_str(" ╔╗  ╔═╗ ╔═╗ ╔═╗ ╔╦╗\n");
		output.push_str(" ╠╩╗ ║╣  ╠═╣ ╚═╗  ║\n");
		output.push_str(" ╚═╝ ╚═╝ ╩ ╩ ╚═╝  ╩\n");
	}

	fn render_footer(&self, output: &mut String) {
		output.push_str("⌂⌂\n\n");
	}

	pub fn render(&self) -> String {
		let mut output = String::new();

		self.render_header(&mut output);
		writeln!(output, "\x1b[33m▛{}▜ \x1b[39m", "▀▀".repeat(BOARD_WIDTH))
			.unwrap_or_else(|_| panic!("Can't write to string buffer"));
		output.push_str(&self.board.render());
		writeln!(output, "\x1b[33m▙{}▟  \x1b[39m", "▄▄".repeat(BOARD_WIDTH))
			.unwrap_or_else(|_| panic!("Can't write to string buffer"));
		self.render_footer(&mut output);

		output
	}

	pub fn re_render(&self) -> String {
		let top_pos = format!("\x1b[{}F", ANSI_FRAME_HEIGHT + ANSI_BOARD_HEIGHT + ANSI_FRAME_HEIGHT + ANSI_FOOTER_HEIGHT);
		let bottom_pos = format!("\x1b[{}E", ANSI_FRAME_HEIGHT + ANSI_FOOTER_HEIGHT + ANSI_FRAME_HEIGHT);
		let mut output = String::new();

		output.push_str(&top_pos);
		output.push_str(&self.board.render());
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
		Game::new().render_header(&mut output);
		assert_eq!(
			output.lines().count(),
			ANSI_HEADER_HEIGHT,
			"There should be exactly ANSI_HEADER_HEIGHT lines in the header"
		);
	}

	#[test]
	fn board_height_test() {
		assert_eq!(
			Game::new().render().lines().count(),
			ANSI_HEADER_HEIGHT + ANSI_FRAME_HEIGHT + ANSI_BOARD_HEIGHT + ANSI_FRAME_HEIGHT + ANSI_FOOTER_HEIGHT,
			"There should be the right amount of lines in the board"
		);
	}

	#[test]
	fn footer_height_test() {
		let mut output = String::new();
		Game::new().render_footer(&mut output);
		assert_eq!(
			output.lines().count(),
			ANSI_FOOTER_HEIGHT,
			"There should be exactly ANSI_FOOTER_HEIGHT lines in the footer"
		);
	}
}
