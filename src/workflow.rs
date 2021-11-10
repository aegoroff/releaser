extern crate ansi_term;

use std::{thread, time};

use ansi_term::Colour::Green;
use semver::Version;
use vfs::VfsPath;

use crate::new_cargo_config_path;
use crate::version_iter::VersionIter;
use crate::CrateConfig;
use crate::Increment;
use crate::Publisher;
use crate::Vcs;

/// Represents virtual path in a filesystem
/// that keeps real fs path that is root of this
/// virtual path
pub struct VPath<'a> {
    real_path: &'a str,
    virtual_path: VfsPath,
}

impl<'a> VPath<'a> {
    pub fn new(real_path: &'a str, virtual_path: VfsPath) -> Self {
        Self {
            virtual_path,
            real_path,
        }
    }
}

pub trait Release<'a> {
    /// Releases crate or workspace
    /// * `root` - path to folder where crate's or workspace's Cargo.toml located
    /// * `incr` - Version increment (major, minor or patch)
    fn release(&self, root: VPath<'a>, incr: Increment) -> crate::Result<()>;
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

impl<'a, P: Publisher, V: Vcs> Release<'a> for Workspace<P, V> {
    fn release(&self, root: VPath<'a>, incr: Increment) -> crate::Result<()> {
        let crate_conf = new_cargo_config_path(&root.virtual_path).unwrap();

        let mut it = VersionIter::open(&crate_conf)?;
        let version = crate::update_configs(&crate_conf, &mut it, incr)?;

        let ver = commit_version(&self.vcs, root.real_path, version)?;

        let delay_str = format!("{}", self.delay_seconds);
        let delay = time::Duration::from_secs(self.delay_seconds);
        let crates_to_publish = it.topo_sort();
        for (i, publish) in crates_to_publish.iter().enumerate() {
            self.publisher.publish(root.real_path, publish)?;
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

        self.vcs.create_tag(root.real_path, &ver)?;
        self.vcs.push_tag(root.real_path, &ver)?;

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

impl<'a, P: Publisher, V: Vcs> Release<'a> for Crate<P, V> {
    fn release(&self, root: VPath<'a>, incr: Increment) -> crate::Result<()> {
        let crate_conf = new_cargo_config_path(&root.virtual_path).unwrap();

        let conf = CrateConfig::open(&crate_conf)?;
        let ver = conf.new_version(String::new());
        let version = crate::update_config(&crate_conf, &ver, incr)?;

        let ver = commit_version(&self.vcs, root.real_path, version)?;

        self.publisher.publish_current(root.real_path)?;

        self.vcs.create_tag(root.real_path, &ver)?;
        self.vcs.push_tag(root.real_path, &ver)?;

        Ok(())
    }
}

fn commit_version(vcs: &impl Vcs, path: &str, version: Version) -> crate::Result<String> {
    let ver = format!("v{}", version);
    let commit_msg = format!("changelog: {}", &ver);
    vcs.commit(path, &commit_msg)?;
    Ok(ver)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MockVcs;
    use crate::{MockPublisher, CARGO_CONFIG};
    use mockall::predicate::*;
    use rstest::*;
    use spectral::prelude::*;
    use vfs::MemoryFS;

    #[rstest]
    fn release_workspace(root: VfsPath) {
        // Arrange
        let mut mock_pub = MockPublisher::new();
        let mut mock_vcs = MockVcs::new();

        mock_vcs
            .expect_commit()
            .with(eq("/x"), eq("changelog: v0.2.0"))
            .times(1)
            .returning(|_, _| Ok(()));

        mock_pub
            .expect_publish()
            .with(eq("/x"), eq("solp"))
            .times(1)
            .returning(|_, _| Ok(()));

        mock_pub
            .expect_publish()
            .with(eq("/x"), eq("solv"))
            .times(1)
            .returning(|_, _| Ok(()));

        mock_vcs
            .expect_create_tag()
            .with(eq("/x"), eq("v0.2.0"))
            .times(1)
            .returning(|_, _| Ok(()));

        mock_vcs
            .expect_push_tag()
            .with(eq("/x"), eq("v0.2.0"))
            .times(1)
            .returning(|_, _| Ok(()));

        let w = Workspace::new(0, mock_pub, mock_vcs);
        let path = VPath::new("/x", root);

        // Act
        let r = w.release(path, Increment::Minor);

        // Assert
        assert_that!(r).is_ok();
    }

    #[rstest]
    fn release_crate(root: VfsPath) {
        // Arrange
        let mut mock_pub = MockPublisher::new();
        let mut mock_vcs = MockVcs::new();

        mock_vcs
            .expect_commit()
            .with(eq("/x"), eq("changelog: v0.2.0"))
            .times(1)
            .returning(|_, _| Ok(()));

        mock_pub
            .expect_publish_current()
            .with(eq("/x"))
            .times(1)
            .returning(|_| Ok(()));

        mock_vcs
            .expect_create_tag()
            .with(eq("/x"), eq("v0.2.0"))
            .times(1)
            .returning(|_, _| Ok(()));

        mock_vcs
            .expect_push_tag()
            .with(eq("/x"), eq("v0.2.0"))
            .times(1)
            .returning(|_, _| Ok(()));

        let c = Crate::new(mock_pub, mock_vcs);

        let path = VPath::new("/x", root.join("solp").unwrap());

        // Act
        let r = c.release(path, Increment::Minor);

        // Assert
        assert_that!(r).is_ok();
    }

    #[fixture]
    fn root() -> VfsPath {
        let root = VfsPath::new(MemoryFS::new());

        root.join("solv").unwrap().create_dir().unwrap();
        root.join("solp").unwrap().create_dir().unwrap();
        root.join(CARGO_CONFIG)
            .unwrap()
            .create_file()
            .unwrap()
            .write_all(WKS.as_bytes())
            .unwrap();

        let ch_fn = |c: &str, d: &str| {
            let ch_conf = root.join(c).unwrap().join(CARGO_CONFIG).unwrap();
            ch_conf
                .create_file()
                .unwrap()
                .write_all(d.as_bytes())
                .unwrap();
        };

        ch_fn("solv", SOLV);
        ch_fn("solp", SOLP);

        root
    }

    const WKS: &str = r#"
[workspace]

members = [
    "solv",
    "solp",
]
        "#;

    const SOLP: &str = r#"
[package]
name = "solp"
description = "Microsoft Visual Studio solution parsing library"
repository = "https://github.com/aegoroff/solv"
version = "0.1.13"
authors = ["egoroff <egoroff@gmail.com>"]
edition = "2018"
license = "MIT"
workspace = ".."

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[build-dependencies] # <-- We added this and everything after!
lalrpop = "0.19"

[dependencies]
lalrpop-util = "0.19"
regex = "1"
jwalk = "0.6"
phf = { version = "0.8", features = ["macros"] }
itertools = "0.10"

        "#;

    const SOLV: &str = r#"
[package]
name = "solv"
description = "Microsoft Visual Studio solution validator"
repository = "https://github.com/aegoroff/solv"
version = "0.1.13"
authors = ["egoroff <egoroff@gmail.com>"]
edition = "2018"
license = "MIT"
workspace = ".."

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
prettytable-rs = "^0.8"
ansi_term = "0.12"
humantime = "2.1"
clap = "2"
fnv = "1"
solp = { path = "../solp/", version = "0.1.13" }

        "#;
}
