use std::io::Write;
use std::process::{Command, Stdio};

fn main() {
    // Spawn a process. Do not wait for it to return.
    // Process should be mutable if we want to signal it later.
    let mut the_process = Command::new("cargo")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .arg("run")
        .arg("-p")
        .arg("editor")
        .arg("node_a.toml")
        .spawn().ok()
        .expect("Failed to execute the cargo run -p editor command");

    let s = "Hello".to_string();
	write!(the_process.stdin.unwrap(), "{}", s).unwrap();

	// let output = String::from_utf8_lossy(&(the_process.stdout));
	// println!("The PID is: {}", output);

	// println!("The PID is: {}", the_process.id());
}
