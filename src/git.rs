use std::io;
use std::process::{Child, Command};

const TOOL: &str = "git";

pub fn commit(message: &str, path: &str) -> io::Result<Child> {
    Command::new(TOOL)
        .current_dir(path)
        .arg("commit")
        .arg("-a")
        .arg("-m")
        .arg(message)
        .spawn()
}

pub fn create_tag(path: &str, tag: &str) -> io::Result<Child> {
    Command::new(TOOL)
        .current_dir(path)
        .arg("tag")
        .arg(tag)
        .spawn()
}

pub fn push_tag(path: &str, tag: &str) -> io::Result<Child> {
    Command::new(TOOL)
        .current_dir(path)
        .arg("push")
        .arg("origin")
        .arg("tag")
        .arg(tag)
        .spawn()
}
