extern crate ansi_term;

use std::{thread, time};

use ansi_term::Colour::Green;
use semver::Version;
use vfs::{VfsPath};

use crate::{git, new_cargo_config_path};
use crate::version_iter::VersionIter;
use crate::Increment;
use crate::{CrateConfig};
use crate::cargo::Publisher;

pub trait Release {
    /// Releases crate or workspace
    /// * `root` - path to folder where crate's or workspace's Cargo.toml located
    /// * `incr` - Version increment (major, minor or patch)
    fn release(&self, root: VfsPath, incr: Increment) -> crate::Result<()>;
}

pub struct Workspace<P: Publisher> {
    delay_seconds: u64,
    publisher: P,
}

impl<P: Publisher> Workspace<P> {
    pub fn new(delay_seconds: u64, publisher: P) -> Self {
        Self { delay_seconds, publisher }
    }
}

impl<P: Publisher> Release for Workspace<P> {
    fn release(&self, root: VfsPath, incr: Increment) -> crate::Result<()> {
        let crate_conf = new_cargo_config_path(&root).unwrap();

        let mut it = VersionIter::open(&crate_conf)?;
        let version = crate::update_configs(&crate_conf, &mut it, incr)?;

        let ver = commit_version(root.as_str(), version)?;

        let delay_str = format!("{}", self.delay_seconds);
        let delay = time::Duration::from_secs(self.delay_seconds);
        let crates_to_publish = it.topo_sort();
        for (i, publish) in crates_to_publish.iter().enumerate() {
            self.publisher.publish(root.as_str(), publish)?;
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

        git::create_tag(root.as_str(), &ver)?;
        git::push_tag(root.as_str(), &ver)?;

        Ok(())
    }
}

#[derive(Default)]
pub struct Crate<P: Publisher> {
    publisher: P,
}

impl<P: Publisher> Crate<P> {
    pub fn new(publisher: P) -> Self {
        Self { publisher }
    }
}

impl<P: Publisher> Release for Crate<P> {
    fn release(&self, root: VfsPath, incr: Increment) -> crate::Result<()> {
        let crate_conf = new_cargo_config_path(&root).unwrap();

        let conf = CrateConfig::open(&crate_conf)?;
        let ver = conf.new_version(String::new());
        let version = crate::update_config(&crate_conf, &ver, incr)?;

        let ver = commit_version(root.as_str(), version)?;

        self.publisher.publish_current(root.as_str())?;

        git::create_tag(root.as_str(), &ver)?;
        git::push_tag(root.as_str(), &ver)?;

        Ok(())
    }
}

fn commit_version(path: &str, version: Version) -> crate::Result<String> {
    let ver = format!("v{}", version);
    let commit_msg = format!("changelog: {}", &ver);
    git::commit(path, &commit_msg)?;
    Ok(ver)
}
