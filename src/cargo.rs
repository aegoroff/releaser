use std::io;
use std::process::{Child, Command};

const TOOL: &str = "cargo";

pub fn publish(path: &str) -> io::Result<Child> {
    Command::new(TOOL)
        .current_dir(path)
        .arg("publish")
        .arg("--manifest-path")
        .arg(path)
        .spawn()
}
