use std::{
	env,
	io::{BufReader, Write},
	process::{Command, Stdio},
	thread,
	time::Duration,
};

mod common;

#[cfg(test)]
mod test {
	use super::{common::helper::*, *};

	#[test]
	fn help_pagination_test() {
		let binary_path = env!("CARGO_BIN_EXE_beast");

		let mut child = Command::new(binary_path)
			.stdin(Stdio::piped())
			.stdout(Stdio::piped())
			.spawn()
			.expect("Failed to spawn game process");

		let mut child_stdin = child.stdin.take().expect("Failed to open child's stdin");
		let child_stdout = child.stdout.take().expect("Failed to open child's stdout");
		let mut reader = BufReader::new(child_stdout);

		let output = get_output(&mut reader, 36);

		assert!(
			output.contains("Faithfully recreated from the work of"),
			"Should contain intro text in output:\n\"{output}\""
		);

		// open help
		child_stdin.write_all(b"h").expect("Failed to write 'h' to child's stdin");
		child_stdin.flush().expect("Failed to flush stdin");
		thread::sleep(Duration::from_millis(100));
		let output = get_output(&mut reader, 32);

		assert!(output.contains("GENERAL"), "Should contain help page one heading in output:\n\"{output}\"");
		assert!(output.contains("● ○ ○"), "Should contain help page one pagination in output:\n\"{output}\"");

		// move to next page
		child_stdin.write_all(b"\x1B[C").expect("Failed to write '→' to child's stdin");
		child_stdin.flush().expect("Failed to flush stdin");
		thread::sleep(Duration::from_millis(100));
		let output = get_output(&mut reader, 17);

		assert!(output.contains("ENEMIES"), "Should contain help page two heading in output:\n\"{output}\"");
		assert!(output.contains("○ ● ○"), "Should contain help page two pagination in output:\n\"{output}\"");

		// move to next page
		child_stdin.write_all(b"\x1B[C").expect("Failed to write '→' to child's stdin");
		child_stdin.flush().expect("Failed to flush stdin");
		thread::sleep(Duration::from_millis(100));
		let output = get_output(&mut reader, 17);

		assert!(output.contains("SCORING"), "Should contain help page three heading in output:\n\"{output}\"");
		assert!(output.contains("○ ○ ●"), "Should contain help page three pagination in output:\n\"{output}\"");

		// move to next page
		child_stdin.write_all(b"\x1B[C").expect("Failed to write '→' to child's stdin");
		child_stdin.flush().expect("Failed to flush stdin");
		thread::sleep(Duration::from_millis(100));
		let output = get_output(&mut reader, 30);

		assert!(output.contains("GENERAL"), "Should contain help page one heading in output:\n\"{output}\"");
		assert!(output.contains("● ○ ○"), "Should contain help page one pagination in output:\n\"{output}\"");

		// move to previous page
		child_stdin.write_all(b"\x1B[D").expect("Failed to write '←' to child's stdin");
		child_stdin.flush().expect("Failed to flush stdin");
		thread::sleep(Duration::from_millis(100));
		let output = get_output(&mut reader, 17);

		assert!(output.contains("SCORING"), "Should contain help page three heading in output:\n\"{output}\"");
		assert!(output.contains("○ ○ ●"), "Should contain help page three pagination in output:\n\"{output}\"");

		// quit program
		child_stdin.write_all(b"q").expect("Failed to write 'q' to child's stdin");
		child_stdin.flush().expect("Failed to flush stdin");
		thread::sleep(Duration::from_millis(100));
		let output = get_output(&mut reader, 2);

		assert!(output.contains("Bye..."), "Expected quit message not found in output:\n\"{output}\"");

		child.wait().expect("Failed to wait on child");
	}
}
