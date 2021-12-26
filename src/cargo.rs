use crate::{PublishOptions, Publisher};
use std::io;
use std::path::PathBuf;
use std::process::Command;

const TOOL: &str = "cargo";

#[derive(Default)]
pub struct Cargo {}

impl Publisher for Cargo {
    fn publish<'a>(&self, path: &str, options: PublishOptions) -> io::Result<()> {
        let mut process = Command::new(TOOL);
        let child = process.current_dir(path).arg("publish");

        if let Some(crt) = options.crate_to_publish {
            let root = PathBuf::from(crt);
            let manifest_path = root.join(crate::CARGO_CONFIG);
            child.arg("--manifest-path").arg(manifest_path);
        }

        if options.all_features {
            child.arg("--all-features");
        }

        child.spawn()?.wait()?;
        Ok(())
    }
}
