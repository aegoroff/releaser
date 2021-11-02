use crate::Publisher;
use std::io;
use std::path::PathBuf;
use std::process::Command;

const TOOL: &str = "cargo";

#[derive(Default)]
pub struct Cargo {}

impl Publisher for Cargo {
    fn publish(&self, path: &str, crt: &str) -> io::Result<()> {
        let root = PathBuf::from(crt);
        let manifest_path = root.join(crate::CARGO_CONFIG);

        let mut child = Command::new(TOOL)
            .current_dir(path)
            .arg("publish")
            .arg("--manifest-path")
            .arg(manifest_path)
            .spawn()?;
        child.wait()?;
        Ok(())
    }

    fn publish_current(&self, path: &str) -> io::Result<()> {
        let mut child = Command::new(TOOL)
            .current_dir(path)
            .arg("publish")
            .spawn()?;
        child.wait()?;
        Ok(())
    }
}
