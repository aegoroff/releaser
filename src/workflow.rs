use crate::git;
use crate::{cargo, CrateConfig, CARGO_CONFIG};
use crate::{Increment, VersionIter};
use ansi_term::Colour::Green;
use semver::Version;
use std::path::PathBuf;
use std::{thread, time};
use vfs::PhysicalFS;

extern crate ansi_term;

pub trait Release {
    fn release(&self, path: &str, incr: Increment) -> crate::Result<()>;
}

pub struct Workspace {
    delay_seconds: u64,
}

impl Workspace {
    pub fn new(delay_seconds: u64) -> Self {
        Self { delay_seconds }
    }
}

impl Release for Workspace {
    fn release(&self, path: &str, incr: Increment) -> crate::Result<()> {
        let root_path = PathBuf::from(path);
        let root = PhysicalFS::new(root_path);

        let mut it = VersionIter::open("/", &root)?;
        let version = crate::update_configs(&root, &mut it, incr)?;

        let ver = commit_version(path, version)?;

        let delay_str = format!("{}", self.delay_seconds);
        let delay = time::Duration::from_secs(self.delay_seconds);
        let crates_to_publish = it.topo_sort();
        for (i, publish) in crates_to_publish.iter().enumerate() {
            cargo::publish(path, publish)?;
            // delay needed between crates to avoid publish failure in case of dependencies
            // crates.io index dont updated instantly
            if i < crates_to_publish.len() - 1 {
                println!(
                    " Waiting {} seconds after publish {} ...",
                    Green.bold().paint(&delay_str),
                    Green.bold().paint(publish)
                );
                thread::sleep(delay);
            }
        }

        git::create_tag(path, &ver)?;
        git::push_tag(path, &ver)?;

        Ok(())
    }
}

pub struct Crate {}

impl Crate {
    pub fn new() -> Self {
        Self {}
    }
}

impl Release for Crate {
    fn release(&self, path: &str, incr: Increment) -> crate::Result<()> {
        let root_path = PathBuf::from(path);
        let root = PhysicalFS::new(root_path);
        let crate_path = PathBuf::from("/").join(CARGO_CONFIG);
        let crate_path = crate_path.to_str().unwrap_or_default();

        let conf = CrateConfig::open(&root, crate_path)?;
        let ver = conf.new_version(crate_path);
        let version = crate::update_config(&root, &ver, incr)?;

        let ver = commit_version(path, version)?;

        cargo::publish_current(path)?;

        git::create_tag(path, &ver)?;
        git::push_tag(path, &ver)?;

        Ok(())
    }
}

fn commit_version(path: &str, version: Version) -> crate::Result<String> {
    let ver = format!("v{}", version);
    let commit_msg = format!("New release {}", &ver);
    git::commit(&commit_msg, path)?;
    Ok(ver)
}
