extern crate ansi_term;

use std::{thread, time};

use ansi_term::Colour::Green;
use semver::Version;
use vfs::{FileSystem, PhysicalFS, VfsPath, VfsResult};

use crate::git;
use crate::{cargo, CrateConfig, CARGO_CONFIG};
use crate::{Increment, VersionIter};
use std::path::PathBuf;

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
        let conf_fs = PhysicalFS::new(PathBuf::from(path));
        let crate_conf = Crate::open(conf_fs).unwrap();

        let mut it = VersionIter::open(&crate_conf)?;
        let version = crate::update_configs(&crate_conf, &mut it, incr)?;

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

    pub fn open(fs: impl FileSystem) -> VfsResult<VfsPath> {
        let p: VfsPath = fs.into();
        p.join(CARGO_CONFIG)
    }
}

impl Default for Crate {
    fn default() -> Self {
        Self::new()
    }
}

impl Release for Crate {
    fn release(&self, path: &str, incr: Increment) -> crate::Result<()> {
        let conf_fs = PhysicalFS::new(PathBuf::from(path));
        let crate_conf = Crate::open(conf_fs).unwrap();

        let conf = CrateConfig::open(&crate_conf)?;
        let ver = conf.new_version(String::new());
        let version = crate::update_config(&crate_conf, &ver, incr)?;

        let ver = commit_version(path, version)?;

        cargo::publish_current(path)?;

        git::create_tag(path, &ver)?;
        git::push_tag(path, &ver)?;

        Ok(())
    }
}

fn commit_version(path: &str, version: Version) -> crate::Result<String> {
    let ver = format!("v{}", version);
    let commit_msg = format!("changelog: {}", &ver);
    git::commit(path, &commit_msg)?;
    Ok(ver)
}
