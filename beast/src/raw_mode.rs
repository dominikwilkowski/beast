//! this module contains a struct that sets the shell into raw mode and returns back normal after it's droped

use std::{io, os::raw::c_int, process::Command};

pub struct RawMode;

impl RawMode {
	pub fn enter() -> io::Result<Self> {
		Command::new("stty").arg("-icanon").arg("-echo").spawn()?.wait()?;
		print!("\x1b[?25l"); // hide cursor
		Ok(RawMode)
	}
}

impl Drop for RawMode {
	fn drop(&mut self) {
		let _ = Command::new("stty").arg("icanon").arg("echo").spawn().and_then(|mut c| c.wait());
		print!("\x1b[?25h"); // show cursor again
	}
}

unsafe extern "C" {
	fn signal(sig: c_int, handler: extern "C" fn(c_int)) -> extern "C" fn(c_int);
}

const SIGINT: c_int = 2;

extern "C" fn handle_sigint(_sig: c_int) {
	print!("\x1b[?25h"); // show cursor again
	let _ = Command::new("stty").arg("icanon").arg("echo").spawn().and_then(|mut c| c.wait());
	std::process::exit(0);
}

pub fn install_raw_mode_signal_handler() {
	unsafe {
		signal(SIGINT, handle_sigint);
	}
}
