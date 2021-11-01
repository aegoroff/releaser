use std::io;
use std::process::Command;

const TOOL: &str = "git";

pub fn commit(path: &str, message: &str) -> io::Result<()> {
    let mut child = Command::new(TOOL)
        .current_dir(path)
        .arg("commit")
        .arg("-a")
        .arg("-m")
        .arg(message)
        .spawn()?;
    child.wait()?;
    Ok(())
}

pub fn create_tag(path: &str, tag: &str) -> io::Result<()> {
    let mut child = Command::new(TOOL)
        .current_dir(path)
        .arg("tag")
        .arg(tag)
        .spawn()?;
    child.wait()?;
    Ok(())
}

pub fn push_tag(path: &str, tag: &str) -> io::Result<()> {
    let mut child = Command::new(TOOL)
        .current_dir(path)
        .arg("push")
        .arg("origin")
        .arg("tag")
        .arg(tag)
        .spawn()?;
    child.wait()?;
    Ok(())
}
