use std::io;
use std::process::{Command};

const TOOL: &str = "cargo";

pub fn publish(path: &str) -> io::Result<()> {
    let mut child = Command::new(TOOL)
        .current_dir(path)
        .arg("publish")
        .arg("--manifest-path")
        .arg(path)
        .spawn()?;
    child.wait()?;
    Ok(())
}
