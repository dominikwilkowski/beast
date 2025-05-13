//! this module contains helper function for raw mode and terminal size using stty

use std::{
	fs::File,
	io,
	os::raw::c_int,
	process::{Command, Stdio},
};

/// the raw mode struct uses the drop trait to restore the terminal state into cooked mode
pub struct RawMode;

impl RawMode {
	/// this method enters the terminal into raw mode
	pub fn enter() -> io::Result<Self> {
		Command::new("stty").arg("-icanon").arg("-echo").spawn()?.wait()?;
		print!("\x1b[?25l"); // hide cursor
		Ok(Self)
	}
}

impl Drop for RawMode {
	/// this method restores the terminal state into cooked mode
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

/// this method installs a signal handler for SIGINT that restores the terminal state into cooked mode
pub fn install_raw_mode_signal_handler() {
	unsafe {
		signal(SIGINT, handle_sigint);
	}
}

/// a function to test if stty is available
pub fn has_stty() -> bool {
	Command::new("stty").arg("size").stdout(Stdio::null()).stderr(Stdio::null()).status().is_ok()
}

/// a function to get the terminal size
pub fn terminal_size() -> io::Result<(usize, usize)> {
	let tty = File::open("/dev/tty")?;
	let output = Command::new("stty").arg("size").stdin(tty).output()?;

	if !output.status.success() {
		return Err(io::Error::new(io::ErrorKind::Other, "stty failed"));
	}
	let output_string = String::from_utf8_lossy(&output.stdout);
	let mut parts = output_string.split_whitespace();
	let rows = match parts.next() {
		Some(rows) => match rows.parse::<usize>() {
			Ok(rows) => rows,
			Err(_) => {
				return Err(io::Error::new(io::ErrorKind::Other, "failed to parse rows"));
			},
		},
		None => {
			return Err(io::Error::new(io::ErrorKind::Other, "failed to parse rows"));
		},
	};

	let columns = match parts.next() {
		Some(columns) => match columns.parse::<usize>() {
			Ok(columns) => columns,
			Err(_) => {
				return Err(io::Error::new(io::ErrorKind::Other, "failed to parse columns"));
			},
		},
		None => {
			return Err(io::Error::new(io::ErrorKind::Other, "failed to parse columns"));
		},
	};

	Ok((columns, rows))
}
