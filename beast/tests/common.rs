#[cfg(test)]
pub mod helper {
	use std::io::{BufRead, BufReader};

	pub fn get_output(reader: &mut BufReader<std::process::ChildStdout>, height: usize) -> String {
		let mut help_output = String::new();
		for _ in 0..height {
			let mut line = String::new();
			if reader.read_line(&mut line).expect("Failed to read line") == 0 {
				break;
			}
			help_output.push_str(&line.replace("\x1b[34F", ""));
		}

		help_output
	}
}
