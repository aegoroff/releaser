use std::io;
use std::process::Command;

use crate::Vcs;

const TOOL: &str = "git";

#[derive(Default)]
pub struct Git;

impl Vcs for Git {
    fn commit(&self, path: &str, message: &str) -> io::Result<()> {
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

    fn create_tag(&self, path: &str, tag: &str) -> io::Result<()> {
        let mut child = Command::new(TOOL)
            .current_dir(path)
            .arg("tag")
            .arg(tag)
            .spawn()?;
        child.wait()?;
        Ok(())
    }

    fn push_tag(&self, path: &str, tag: &str) -> io::Result<()> {
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
}
