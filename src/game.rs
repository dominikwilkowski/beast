use std::{
	io::{self, Read},
	sync::mpsc,
	thread,
	// time::{Duration, Instant},
};

use crate::{
	BOARD_HEIGHT, Level,
	board::Board,
	movement::{Dir, move_player},
	raw_mode::{RawMode, install_raw_mode_signal_handler},
};

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
								move_player(&mut self.board, Dir::Up);
								print!("{}", self.re_render());
							},
							b'C' => {
								move_player(&mut self.board, Dir::Right);
								print!("{}", self.re_render());
							},
							b'B' => {
								move_player(&mut self.board, Dir::Down);
								print!("{}", self.re_render());
							},
							b'D' => {
								move_player(&mut self.board, Dir::Left);
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

	fn render_header() {}
	fn render_footer() {}
	pub fn render() {}

	pub fn re_render(&self) -> String {
		let reset_pos = format!("\x1b[{}F", BOARD_HEIGHT + 1);
		let mut output = String::new();

		output.push_str(&reset_pos);
		output.push_str(&self.board.render_full());
		output
	}
}
