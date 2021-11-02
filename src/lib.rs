#[macro_use]
extern crate handlebars;
extern crate hex;
extern crate itertools;
extern crate petgraph;
extern crate semver;
extern crate serde;
extern crate sha2;
extern crate toml;
extern crate toml_edit;
extern crate vfs;

use std::collections::HashMap;
use std::io;

use semver::{BuildMetadata, Prerelease, Version};
use serde::Deserialize;
use toml_edit::{value, Document};
use vfs::{VfsPath, VfsResult};

pub mod brew;
pub mod cargo;
pub mod git;
pub mod hash;
mod pkg;
mod resource;
pub mod scoop;
mod version_iter;
pub mod workflow;

#[cfg(test)] // <-- not needed in integration tests
extern crate spectral;

#[cfg(test)] // <-- not needed in integration tests
#[macro_use]
extern crate table_test;

#[cfg(test)] // <-- not needed in integration tests
#[macro_use]
extern crate mockall;

#[cfg(test)]
use mockall::{automock, predicate::*};

pub type AnyError = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, AnyError>;

const CARGO_CONFIG: &str = "Cargo.toml";
const VERSION: &str = "version";
const PACK: &str = "package";
const DEPS: &str = "dependencies";

#[cfg_attr(test, automock)]
pub trait Publisher {
    fn publish(&self, path: &str, crt: &str) -> io::Result<()>;
    fn publish_current(&self, path: &str) -> io::Result<()>;
}

#[cfg_attr(test, automock)]
pub trait Vcs {
    fn commit(&self, path: &str, message: &str) -> io::Result<()>;
    fn create_tag(&self, path: &str, tag: &str) -> io::Result<()>;
    fn push_tag(&self, path: &str, tag: &str) -> io::Result<()>;
}

pub fn update_configs<I>(path: &VfsPath, iter: &mut I, incr: Increment) -> Result<Version>
where
    I: Iterator<Item = CrateVersion>,
{
    let result = Version::parse("0.0.0")?;

    let result = iter
        .by_ref()
        .map(|config| update_config(path, &config, incr))
        .filter(|v| v.is_ok())
        .map(|r| r.unwrap())
        .fold(result, |r, v| r.max(v));

    Ok(result)
}

pub fn update_config(path: &VfsPath, version: &CrateVersion, incr: Increment) -> Result<Version> {
    let working_config_path: &VfsPath;
    let member_config_path: VfsPath;
    if version.path.is_empty() {
        working_config_path = path;
    } else {
        let parent = path.parent().unwrap();
        member_config_path = parent.join(&version.path)?.join(CARGO_CONFIG)?;
        working_config_path = &member_config_path;
    }

    let mut file = working_config_path.open_file()?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let mut doc = content.parse::<Document>()?;

    let mut result = Version::parse("0.0.0")?;

    for place in &version.places {
        match place {
            Place::Package(ver) => {
                let v = increment(ver, incr)?;
                result = result.max(v);
                doc[PACK][VERSION] = value(result.to_string());
            }
            Place::Dependency(n, ver) => {
                let v = increment(ver, incr)?;
                result = result.max(v);
                doc[DEPS][n][VERSION] = value(result.to_string());
            }
        }
    }

    let mut f = working_config_path.create_file()?;
    let changed = doc.to_string();
    f.write_all(changed.as_bytes())?;
    Ok(result)
}

fn increment(v: &str, i: Increment) -> Result<Version> {
    let mut v = Version::parse(v)?;
    match i {
        Increment::Major => increment_major(&mut v),
        Increment::Minor => increment_minor(&mut v),
        Increment::Patch => increment_patch(&mut v),
    }
    Ok(v)
}

pub fn new_cargo_config_path(root: &VfsPath) -> VfsResult<VfsPath> {
    root.join(CARGO_CONFIG)
}

fn increment_patch(v: &mut Version) {
    v.patch += 1;
    v.pre = Prerelease::EMPTY;
    v.build = BuildMetadata::EMPTY;
}

fn increment_minor(v: &mut Version) {
    v.minor += 1;
    v.patch = 0;
    v.pre = Prerelease::EMPTY;
    v.build = BuildMetadata::EMPTY;
}

fn increment_major(v: &mut Version) {
    v.major += 1;
    v.minor = 0;
    v.patch = 0;
    v.pre = Prerelease::EMPTY;
    v.build = BuildMetadata::EMPTY;
}

#[derive(Deserialize)]
struct WorkspaceConfig {
    workspace: Workspace,
}

#[derive(Deserialize)]
struct Workspace {
    members: Vec<String>,
}

#[derive(Deserialize, Default)]
struct CrateConfig {
    package: Package,
    dependencies: HashMap<String, Dependency>,
}

impl CrateConfig {
    pub fn open(path: &VfsPath) -> Result<Self> {
        let mut file = path.open_file()?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let conf: CrateConfig = toml::from_str(&content).unwrap();
        Ok(conf)
    }

    pub fn new_version(&self, path: String) -> CrateVersion {
        let places = vec![Place::Package(self.package.version.clone())];

        CrateVersion { places, path }
    }
}

#[derive(Deserialize, Default)]
struct Package {
    name: String,
    version: String,
    description: Option<String>,
    license: Option<String>,
    homepage: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum Dependency {
    Plain(String),
    Object(HashMap<String, Dependency>),
    List(Vec<Dependency>),
}

#[derive(Debug, Default)]
pub struct CrateVersion {
    path: String,
    places: Vec<Place>,
}

/// Place defines where to find version
#[derive(Debug)]
pub enum Place {
    /// Find version in package metadata (i.e. `package` section)
    Package(String),
    /// Find version in dependencies (i.e. `dependencies` section)
    Dependency(String, String),
}

#[derive(Copy, Clone, Debug)]
pub enum Increment {
    Major,
    Minor,
    Patch,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml_parse_workspace() {
        // Arrange

        // Act
        let cfg: WorkspaceConfig = toml::from_str(WKS).unwrap();

        // Assert
        assert_eq!(2, cfg.workspace.members.len());
    }

    #[test]
    fn toml_parse_crate() {
        // Arrange

        // Act
        let cfg: CrateConfig = toml::from_str(SOLV).unwrap();

        // Assert
        assert_eq!("solv", cfg.package.name);
        assert_eq!("0.1.13", cfg.package.version);
        assert_eq!(6, cfg.dependencies.len());
        let solp = &cfg.dependencies["solp"];
        if let Dependency::Object(o) = solp {
            assert_eq!(2, o.len());
            assert!(o.contains_key(VERSION));
            assert!(o.contains_key("path"));
        }
    }

    const WKS: &str = r#"
[workspace]

members = [
    "solv",
    "solp",
]
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
