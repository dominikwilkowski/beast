use std::{
	io::{self, Read},
	process::Command,
	sync::mpsc,
	thread,
	time::{Duration, Instant},
};

use crate::{
	board::Board,
	movement::{move_player, Dir},
	Level,
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
				match byte as char {
					'q' => {
						println!("Bye...");
						break;
					},
					x => {
						move_player(&mut self.board, Dir::Up);
						println!("key={x:?}");
					},
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

	pub fn render() {}
	fn render_header() {}
	fn render_footer() {}
}
