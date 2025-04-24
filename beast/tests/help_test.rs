//! the end to end test for the help module

use std::{
	env,
	io::{BufReader, Write},
	process::{Child, Command, Stdio},
	thread,
	time::Duration,
};

mod common;

struct ChildGuard {
	child: Option<Child>,
}

impl ChildGuard {
	fn new(child: Child) -> Self {
		Self { child: Some(child) }
	}

	fn child_mut(&mut self) -> &mut Child {
		self.child.as_mut().expect("Child already taken")
	}
}

impl Drop for ChildGuard {
	fn drop(&mut self) {
		if let Some(mut child) = self.child.take() {
			let _ = child.kill();
			let _ = child.wait();
		}
	}
}

#[cfg(test)]
mod test {
	use super::{common::*, *};

	#[test]
	fn help_pagination_test() {
		let binary_path = env!("CARGO_BIN_EXE_beast");

		let child = Command::new(binary_path)
			.stdin(Stdio::piped())
			.stdout(Stdio::piped())
			.spawn()
			.expect("Failed to spawn game process");
		let mut child_guard = ChildGuard::new(child);

		let mut child_stdin = child_guard.child_mut().stdin.take().expect("Failed to open child's stdin");
		let child_stdout = child_guard.child_mut().stdout.take().expect("Failed to open child's stdout");
		let mut reader = BufReader::new(child_stdout);

		let output = helper::get_output(&mut reader, 36);

		assert!(
			output.contains("Faithfully recreated from the work of"),
			"Should contain intro text in output:\n\"{output}\""
		);

		// open help
		child_stdin.write_all(b"h").expect("Failed to write 'h' to child's stdin");
		child_stdin.flush().expect("Failed to flush stdin");
		thread::sleep(Duration::from_millis(100));
		let output = helper::get_output(&mut reader, 32);

		assert!(output.contains("GENERAL"), "Should contain help page one heading in output:\n\"{output}\"");
		assert!(output.contains("● ○ ○"), "Should contain help page one pagination in output:\n\"{output}\"");

		// move to next page
		child_stdin.write_all(b"\x1B[C").expect("Failed to write '→' to child's stdin");
		child_stdin.flush().expect("Failed to flush stdin");
		thread::sleep(Duration::from_millis(100));
		let output = helper::get_output(&mut reader, 17);

		assert!(output.contains("ENEMIES"), "Should contain help page two heading in output:\n\"{output}\"");
		assert!(output.contains("○ ● ○"), "Should contain help page two pagination in output:\n\"{output}\"");

		// move to next page
		child_stdin.write_all(b"\x1B[C").expect("Failed to write '→' to child's stdin");
		child_stdin.flush().expect("Failed to flush stdin");
		thread::sleep(Duration::from_millis(100));
		let output = helper::get_output(&mut reader, 17);

		assert!(output.contains("SCORING"), "Should contain help page three heading in output:\n\"{output}\"");
		assert!(output.contains("○ ○ ●"), "Should contain help page three pagination in output:\n\"{output}\"");

		// move to next page
		child_stdin.write_all(b"\x1B[C").expect("Failed to write '→' to child's stdin");
		child_stdin.flush().expect("Failed to flush stdin");
		thread::sleep(Duration::from_millis(100));
		let output = helper::get_output(&mut reader, 30);

		assert!(output.contains("GENERAL"), "Should contain help page one heading in output:\n\"{output}\"");
		assert!(output.contains("● ○ ○"), "Should contain help page one pagination in output:\n\"{output}\"");

		// move to previous page
		child_stdin.write_all(b"\x1B[D").expect("Failed to write '←' to child's stdin");
		child_stdin.flush().expect("Failed to flush stdin");
		thread::sleep(Duration::from_millis(100));
		let output = helper::get_output(&mut reader, 17);

		assert!(output.contains("SCORING"), "Should contain help page three heading in output:\n\"{output}\"");
		assert!(output.contains("○ ○ ●"), "Should contain help page three pagination in output:\n\"{output}\"");

		// quit program
		child_stdin.write_all(b"q").expect("Failed to write 'q' to child's stdin");
		child_stdin.flush().expect("Failed to flush stdin");
		thread::sleep(Duration::from_millis(100));
		let output = helper::get_output(&mut reader, 2);

		assert!(output.contains("Bye..."), "Expected quit message not found in output:\n\"{output}\"");

		drop(child_stdin);
	}

	#[test]
	fn help_opening_playing_test() {
		let binary_path = env!("CARGO_BIN_EXE_beast");

		let child = Command::new(binary_path)
			.stdin(Stdio::piped())
			.stdout(Stdio::piped())
			.spawn()
			.expect("Failed to spawn game process");
		let mut child_guard = ChildGuard::new(child);

		let mut child_stdin = child_guard.child_mut().stdin.take().expect("Failed to open child's stdin");
		let child_stdout = child_guard.child_mut().stdout.take().expect("Failed to open child's stdout");
		let mut reader = BufReader::new(child_stdout);

		let output = helper::get_output(&mut reader, 36);

		assert!(
			output.contains("Faithfully recreated from the work of"),
			"Should contain intro text in output:\n\"{output}\""
		);

		// open help
		child_stdin.write_all(b"h").expect("Failed to write 'h' to child's stdin");
		child_stdin.flush().expect("Failed to flush stdin");
		thread::sleep(Duration::from_millis(100));
		let output = helper::get_output(&mut reader, 33);

		assert!(output.contains("GENERAL"), "Should contain help page one heading in output:\n\"{output}\"");
		assert!(output.contains("● ○ ○"), "Should contain help page one pagination in output:\n\"{output}\"");

		// start game
		child_stdin.write_all(b" ").expect("Failed to write ' ' to child's stdin");
		child_stdin.flush().expect("Failed to flush stdin");
		thread::sleep(Duration::from_millis(300));
		let output = helper::get_output(&mut reader, 33);

		assert!(output.contains("Level:"), "Should contain level in footer in output:\n\"{output}\"");
		assert!(output.contains("Beasts:"), "Should contain beasts in footer in output:\n\"{output}\"");
		assert!(output.contains("Lives:"), "Should contain lives in footer in output:\n\"{output}\"");
		assert!(output.contains("Time:"), "Should contain time in footer in output:\n\"{output}\"");
		assert!(output.contains("Score:"), "Should contain score in footer in output:\n\"{output}\"");
		assert!(output.contains("░░"), "Should contain blocks in output:\n\"{output}\"");

		// starting help
		child_stdin.flush().expect("Failed to flush stdin");
		let _output = helper::get_output(&mut reader, 33);
		child_stdin.flush().expect("Failed to flush stdin");
		let _output = helper::get_output(&mut reader, 33);
		child_stdin.write_all(b"h").expect("Failed to write 'h' to child's stdin");
		child_stdin.flush().expect("Failed to flush stdin");
		thread::sleep(Duration::from_millis(100));
		let output = helper::get_output(&mut reader, 31);

		assert!(output.contains("GENERAL"), "Should contain help page one heading after game in output:\n\"{output}\"");
		assert!(output.contains("● ○ ○"), "Should contain help page one pagination after game in output:\n\"{output}\"");

		// quit program
		child_stdin.write_all(b"q").expect("Failed to write 'q' to child's stdin");
		child_stdin.flush().expect("Failed to flush stdin");
		thread::sleep(Duration::from_millis(100));
		let output = helper::get_output(&mut reader, 2);

		assert!(output.contains("Bye..."), "Expected quit message not found in output:\n\"{output}\"");

		child_guard.child_mut().wait().expect("Failed to wait on child");

		drop(child_stdin);
	}
}
