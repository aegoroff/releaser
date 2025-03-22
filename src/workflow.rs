use std::{thread, time};

use console::style;
use semver::Version;
use vfs::VfsPath;

use crate::CrateConfig;
use crate::Increment;
use crate::Publisher;
use crate::Vcs;
use crate::version_iter::VersionIter;
use crate::{PublishOptions, new_cargo_config_path};
use color_eyre::eyre::Result;

/// Represents virtual path in a filesystem
/// that keeps real fs path that is root of this
/// virtual path
pub struct VPath<'a> {
    real_path: &'a str,
    virtual_path: VfsPath,
}

impl<'a> VPath<'a> {
    #[must_use]
    pub fn new(real_path: &'a str, virtual_path: VfsPath) -> Self {
        Self {
            real_path,
            virtual_path,
        }
    }
}

pub trait Release<'a> {
    /// Releases crate or workspace
    /// * `root` - path to folder where crate's or workspace's Cargo.toml located
    /// * `incr` - Version increment (major, minor or patch)
    /// * `all_features` - whether to publish all features i.e. pass --all-features flag to cargo publish
    /// * `no_verify` - whether to verify package tarball before publish i.e. pass --no-verify flag to cargo publish
    fn release(
        &self,
        root: VPath<'a>,
        incr: Increment,
        all_features: bool,
        no_verify: bool,
    ) -> Result<()>;
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
    fn release(
        &self,
        root: VPath<'a>,
        incr: Increment,
        all_features: bool,
        no_verify: bool,
    ) -> Result<()> {
        let crate_conf = new_cargo_config_path(&root.virtual_path)?;

        let mut it = VersionIter::open(&crate_conf)?;
        let version = crate::update_configs(&crate_conf, &mut it, incr)?;

        let ver = commit_version(&self.vcs, root.real_path, &version)?;

        let delay_str = format!("{}", self.delay_seconds);
        let delay = time::Duration::from_secs(self.delay_seconds);
        let crates_to_publish = it.topo_sort();
        for (i, publish) in crates_to_publish.iter().enumerate() {
            let options = PublishOptions {
                crate_to_publish: Some(publish),
                all_features,
                no_verify,
            };
            self.publisher.publish(root.real_path, options)?;
            // delay between crates needed to avoid publish failure
            // because crates.io index aren't updated instantly
            if i < crates_to_publish.len() - 1 {
                println!(
                    " Waiting {} seconds after publish {} ...",
                    style(&delay_str).green().bold(),
                    style(publish).green().bold()
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
    fn release(
        &self,
        root: VPath<'a>,
        incr: Increment,
        all_features: bool,
        no_verify: bool,
    ) -> Result<()> {
        let crate_conf = new_cargo_config_path(&root.virtual_path)?;

        let conf = CrateConfig::open(&crate_conf)?;
        let ver = conf.new_version(String::new());
        let version = crate::update_config(&crate_conf, &ver, incr)?;

        let ver = commit_version(&self.vcs, root.real_path, &version)?;

        let options = PublishOptions {
            crate_to_publish: None,
            all_features,
            no_verify,
        };
        self.publisher.publish(root.real_path, options)?;

        self.vcs.create_tag(root.real_path, &ver)?;
        self.vcs.push_tag(root.real_path, &ver)?;

        Ok(())
    }
}

fn commit_version(vcs: &impl Vcs, path: &str, version: &Version) -> Result<String> {
    let ver = format!("v{version}");
    let commit_msg = format!("changelog: {ver}");
    vcs.commit(path, &commit_msg)?;
    Ok(ver)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_in_result)]
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::MockVcs;
    use crate::{CARGO_CONFIG, MockPublisher};
    use mockall::predicate::{eq, str};
    use rstest::{fixture, rstest};
    use vfs::MemoryFS;

    #[rstest]
    #[case::all_features(true)]
    #[case::default_features(false)]
    #[trace]
    fn release_workspace(root: VfsPath, #[case] all_features: bool) {
        // Arrange
        let mut mock_pub = MockPublisher::new();
        let mut mock_vcs = MockVcs::new();

        mock_vcs
            .expect_commit()
            .with(eq("/x"), eq("changelog: v0.2.0"))
            .times(1)
            .returning(|_, _| Ok(()));

        let solp_options: PublishOptions = PublishOptions {
            crate_to_publish: Some("solp"),
            all_features,
            no_verify: false,
        };
        mock_pub
            .expect_publish()
            .withf(move |p, o| p == "/x" && *o == solp_options)
            .times(1)
            .returning(|_, _| Ok(()));

        let solv_options: PublishOptions = PublishOptions {
            crate_to_publish: Some("solv"),
            all_features,
            no_verify: false,
        };
        mock_pub
            .expect_publish()
            .withf(move |p, o| p == "/x" && *o == solv_options)
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
        let r = w.release(path, Increment::Minor, all_features, false);

        // Assert
        assert!(r.is_ok());
    }

    #[rstest]
    #[case::all_features(true)]
    #[case::default_features(false)]
    #[trace]
    fn release_crate(root: VfsPath, #[case] all_features: bool) {
        // Arrange
        let mut mock_pub = MockPublisher::new();
        let mut mock_vcs = MockVcs::new();

        mock_vcs
            .expect_commit()
            .with(eq("/x"), eq("changelog: v0.2.0"))
            .times(1)
            .returning(|_, _| Ok(()));

        let options: PublishOptions = PublishOptions {
            crate_to_publish: None,
            all_features,
            no_verify: false,
        };
        mock_pub
            .expect_publish()
            .withf(move |p, o| p == "/x" && *o == options)
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

        let c = Crate::new(mock_pub, mock_vcs);

        let path = VPath::new("/x", root.join("solp").unwrap());

        // Act
        let r = c.release(path, Increment::Minor, all_features, false);

        // Assert
        assert!(r.is_ok());
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
