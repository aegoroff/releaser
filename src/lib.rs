#[macro_use]
extern crate handlebars;
extern crate hex;
extern crate petgraph;
extern crate semver;
extern crate serde;
extern crate sha2;
extern crate toml;
extern crate toml_edit;
extern crate vfs;

use std::collections::HashMap;
use std::io;

use clap::ValueEnum;
use error::FileError;
#[cfg(test)]
use mockall::{automock, predicate::*};
use semver::{BuildMetadata, Prerelease, Version};
use serde::Deserialize;

use toml_edit::{value, Document};
use vfs::VfsPath;

pub mod brew;
pub mod cargo;
pub mod error;
pub mod git;
pub mod hash;
mod pkg;
mod resource;
pub mod scoop;
mod version_iter;
pub mod workflow;

#[cfg(test)] // <-- not needed in integration tests
#[macro_use]
extern crate mockall;

#[cfg(test)] // <-- not needed in integration tests
extern crate rstest;

pub type AnyError = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, AnyError>;

const CARGO_CONFIG: &str = "Cargo.toml";
const VERSION: &str = "version";
const PACK: &str = "package";
const DEPS: &str = "dependencies";

#[derive(Default, Eq, PartialEq, Debug)]
pub struct PublishOptions<'a> {
    pub crate_to_publish: Option<&'a str>,
    pub all_features: bool,
    pub no_verify: bool,
}

#[cfg_attr(test, automock)]
pub trait Publisher {
    fn publish<'a>(&'a self, path: &'a str, options: PublishOptions<'a>) -> io::Result<()>;
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
        .filter_map(|v| v.ok())
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
        member_config_path = match match parent.join(&version.path) {
            Ok(it) => it,
            Err(err) => return Err(Box::new(FileError::from(err))),
        }
        .join(CARGO_CONFIG)
        {
            Ok(it) => it,
            Err(err) => return Err(Box::new(FileError::from(err))),
        };
        working_config_path = &member_config_path;
    }

    let mut file = match working_config_path.open_file() {
        Ok(it) => it,
        Err(err) => return Err(Box::new(FileError::from(err))),
    };
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

    let mut f = match working_config_path.create_file() {
        Ok(it) => it,
        Err(err) => return Err(Box::new(FileError::from(err))),
    };
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

fn new_cargo_config_path(root: &VfsPath) -> Option<VfsPath> {
    join(root, CARGO_CONFIG)
}

fn join(p: &VfsPath, other: &str) -> Option<VfsPath> {
    match p.join(other) {
        Ok(r) => Some(r),
        Err(_) => None,
    }
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
    dependencies: Option<HashMap<String, Dependency>>,
}

impl CrateConfig {
    pub fn open(path: &VfsPath) -> Result<Self> {
        let mut file = match path.open_file() {
            Ok(it) => it,
            Err(err) => return Err(Box::new(FileError::from(err))),
        };
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let conf: CrateConfig = toml::from_str(&content)?;
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
    Optional(bool),
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Increment {
    Major,
    Minor,
    Patch,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case::patch(Increment::Patch, "0.1.2")]
    #[case::minor(Increment::Minor, "0.2.0")]
    #[case::major(Increment::Major, "1.0.0")]
    #[trace]
    fn increment_tests(#[case] incr: Increment, #[case] expected: &str) {
        // Arrange
        let v = "0.1.1";

        // Act
        let actual = increment(v, incr).unwrap();

        // Assert
        assert_eq!(actual, Version::parse(expected).unwrap());
    }

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
        let deps = cfg.dependencies.unwrap();
        assert_eq!("solv", cfg.package.name);
        assert_eq!("0.1.13", cfg.package.version);
        assert_eq!(6, deps.len());
        let solp = &deps["solp"];
        if let Dependency::Object(o) = solp {
            assert_eq!(2, o.len());
            assert!(o.contains_key(VERSION));
            assert!(o.contains_key("path"));
        }
    }

    #[test]
    fn toml_parse_crate_with_optional_deps() {
        // Arrange
        let conf = r#"[package]
name = "editorconfiger"
version = "0.1.9"
description = "Plain tool to validate and compare .editorconfig files"
authors = ["egoroff <egoroff@gmail.com>"]
keywords = ["editorconfig"]
homepage = "https://github.com/aegoroff/editorconfiger"
edition = "2021"
license = "MIT"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[build-dependencies] # <-- We added this and everything after!
lalrpop = "0.19"

[dependencies]
lalrpop-util  = { version = "0.19", features = ["lexer"] }
regex = "1"
jwalk = "0.6"
aho-corasick = "0.7"
nom = "7"
num_cpus = "1.13.0"

ansi_term = { version = "0.12", optional = true }
prettytable-rs = { version = "^0.8", optional = true }
clap = { version = "2", optional = true }

[dev-dependencies]
table-test = "0.2.1"
spectral = "0.6.0"
rstest = "0.12.0"

[features]
build-binary = ["clap", "ansi_term", "prettytable-rs"]

[[bin]]
name = "editorconfiger"
required-features = ["build-binary"]

[profile.release]
lto = true"#;

        // Act
        let cfg: CrateConfig = toml::from_str(conf).unwrap();

        // Assert
        let deps = cfg.dependencies.unwrap();
        assert_eq!("editorconfiger", cfg.package.name);
        assert_eq!("0.1.9", cfg.package.version);
        assert_eq!(9, deps.len());
        let ansi_term = &deps["ansi_term"];
        if let Dependency::Object(o) = ansi_term {
            assert_eq!(2, o.len());
            assert!(o.contains_key(VERSION));
            assert!(o.contains_key("optional"));
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
