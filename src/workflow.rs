extern crate ansi_term;

use std::{thread, time};

use ansi_term::Colour::Green;
use semver::Version;
use vfs::VfsPath;

use crate::cargo::Publisher;
use crate::git::Vcs;
use crate::new_cargo_config_path;
use crate::version_iter::VersionIter;
use crate::CrateConfig;
use crate::Increment;

pub trait Release {
    /// Releases crate or workspace
    /// * `root` - path to folder where crate's or workspace's Cargo.toml located
    /// * `incr` - Version increment (major, minor or patch)
    fn release(&self, root: VfsPath, incr: Increment) -> crate::Result<()>;
}

pub struct Workspace<P: Publisher, V: Vcs> {
    delay_seconds: u64,
    publisher: P,
    vcs: V,
}

impl<P: Publisher, V: Vcs> Workspace<P, V> {
    pub fn new(delay_seconds: u64, publisher: P, vcs: V) -> Self {
        Self {
            delay_seconds,
            publisher,
            vcs,
        }
    }
}

impl<P: Publisher, V: Vcs> Release for Workspace<P, V> {
    fn release(&self, root: VfsPath, incr: Increment) -> crate::Result<()> {
        let crate_conf = new_cargo_config_path(&root).unwrap();

        let mut it = VersionIter::open(&crate_conf)?;
        let version = crate::update_configs(&crate_conf, &mut it, incr)?;

        let ver = commit_version(&self.vcs, root.as_str(), version)?;

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

        self.vcs.create_tag(root.as_str(), &ver)?;
        self.vcs.push_tag(root.as_str(), &ver)?;

        Ok(())
    }
}

#[derive(Default)]
pub struct Crate<P: Publisher, V: Vcs> {
    publisher: P,
    vcs: V,
}

impl<P: Publisher, V: Vcs> Crate<P, V> {
    pub fn new(publisher: P, vcs: V) -> Self {
        Self { publisher, vcs }
    }
}

impl<P: Publisher, V: Vcs> Release for Crate<P, V> {
    fn release(&self, root: VfsPath, incr: Increment) -> crate::Result<()> {
        let crate_conf = new_cargo_config_path(&root).unwrap();

        let conf = CrateConfig::open(&crate_conf)?;
        let ver = conf.new_version(String::new());
        let version = crate::update_config(&crate_conf, &ver, incr)?;

        let ver = commit_version(&self.vcs, root.as_str(), version)?;

        self.publisher.publish_current(root.as_str())?;

        self.vcs.create_tag(root.as_str(), &ver)?;
        self.vcs.push_tag(root.as_str(), &ver)?;

        Ok(())
    }
}

fn commit_version(vcs: &impl Vcs, path: &str, version: Version) -> crate::Result<String> {
    let ver = format!("v{}", version);
    let commit_msg = format!("changelog: {}", &ver);
    vcs.commit(path, &commit_msg)?;
    Ok(ver)
}
