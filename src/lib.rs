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

use petgraph::algo::DfsSpace;
use petgraph::graphmap::DiGraphMap;
use semver::{BuildMetadata, Prerelease, Version};
use serde::Deserialize;
use toml_edit::{value, Document};
use vfs::VfsPath;

pub mod brew;
mod cargo;
mod git;
pub mod hash;
mod pkg;
mod resource;
pub mod scoop;
pub mod workflow;

#[cfg(test)] // <-- not needed in integration tests
#[macro_use]
extern crate spectral;

#[cfg(test)] // <-- not needed in integration tests
#[macro_use]
extern crate table_test;

pub type AnyError = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, AnyError>;

const CARGO_CONFIG: &str = "Cargo.toml";
const VERSION: &str = "version";
const PACK: &str = "package";
const DEPS: &str = "dependencies";

pub struct VersionIter<'a> {
    search: HashMap<String, usize>,
    members: Vec<String>,
    workspace_config_path: &'a VfsPath,
    graph: DiGraphMap<usize, i32>,
}

impl<'a> VersionIter<'a> {
    pub fn open(path: &'a VfsPath) -> Result<Self> {
        let mut wks_file = path.open_file()?;
        let mut wc = String::new();
        wks_file.read_to_string(&mut wc)?;
        let wks: WorkspaceConfig = toml::from_str(&wc)?;
        let search: HashMap<String, usize> = wks
            .workspace
            .members
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, s)| (s, i))
            .collect();
        let members = wks.workspace.members;

        let graph = DiGraphMap::new();
        Ok(Self {
            search,
            members,
            workspace_config_path: path,
            graph,
        })
    }

    pub fn topo_sort(&self) -> Vec<String> {
        let reverted = self
            .search
            .iter()
            .map(|(k, v)| (*v, k))
            .collect::<HashMap<usize, &String>>();

        let mut space = DfsSpace::new(&self.graph);
        let sorted = petgraph::algo::toposort(&self.graph, Some(&mut space)).unwrap_or_default();

        sorted
            .into_iter()
            .map(|g| *reverted.get(&g).unwrap())
            .cloned()
            .collect()
    }
}

impl<'a> Iterator for VersionIter<'a> {
    type Item = CrateVersion;

    fn next(&mut self) -> Option<Self::Item> {
        let member = self.members.pop()?;
        let root = self.workspace_config_path.parent()?;
        let config_path = root.join(&member).unwrap().join(CARGO_CONFIG);
        let config_path = config_path.unwrap();

        let conf = CrateConfig::open(&config_path).unwrap_or_default();
        if conf.package.version.is_empty() {
            return None;
        }

        let mut item = conf.new_version(member);

        let deps = conf
            .dependencies
            .into_iter()
            .filter(|(n, _)| self.search.contains_key(n))
            .filter_map(|(n, v)| {
                if let Dependency::Object(m) = v {
                    let d = m.get(VERSION)?;
                    if let Dependency::Plain(s) = d {
                        return Some(Place::Dependency(n, s.clone()));
                    }
                }
                None
            });

        item.places.extend(deps);

        let to = self.search.get(&conf.package.name).unwrap();

        for place in item.places.iter() {
            if let Place::Dependency(n, _) = place {
                let from = self.search.get(n).unwrap();
                self.graph.add_edge(*from, *to, -1);
            }
        }

        Some(item)
    }
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

#[derive(Debug)]
pub enum Place {
    Package(String),
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
    use std::path::PathBuf;
    use vfs::{FileSystem, MemoryFS};

    use super::*;
    use spectral::prelude::*;

    #[test]
    fn read_workspace_test() {
        // Arrange
        let fs = new_file_system();
        let root: VfsPath = fs.into();
        let conf = root.join(CARGO_CONFIG).unwrap();
        let it = VersionIter::open(&conf).unwrap();

        // Act
        let versions = it.count();

        // Assert
        assert_that!(versions).is_equal_to(2);
    }

    #[test]
    fn read_empty_workspace_test() {
        // Arrange
        let root_path = PathBuf::from("/");
        let fs = MemoryFS::new();
        fs.create_dir(root_path.to_str().unwrap()).unwrap();
        let root: VfsPath = fs.into();
        let conf = root.join(CARGO_CONFIG).unwrap();

        // Act
        let result = VersionIter::open(&conf);

        // Assert
        assert_that!(result.is_err()).is_true();
    }

    #[test]
    fn update_workspace_version_change_tests() {
        // Arrange
        let cases = vec![
            (Increment::Patch, "0.1.14"),
            (Increment::Minor, "0.2.0"),
            (Increment::Major, "1.0.0"),
        ];

        // Act
        for (validator, input, expected) in table_test!(cases) {
            let fs = new_file_system();
            let root: VfsPath = fs.into();
            let conf = root.join(CARGO_CONFIG).unwrap();
            let mut it = VersionIter::open(&conf).unwrap();
            let actual = update_configs(&conf, &mut it, input).unwrap().to_string();

            validator
                .given(&format!("Increment: {:#?}", input))
                .when("update_configs")
                .then(&format!("it should be {}", expected))
                .assert_eq(expected, &actual);
        }
    }

    #[test]
    fn version_iter_topo_sort_test() {
        // Arrange
        let fs = new_file_system();
        let root: VfsPath = fs.into();
        let conf = root.join(CARGO_CONFIG).unwrap();
        let mut it = VersionIter::open(&conf).unwrap();
        let actual = update_configs(&conf, &mut it, Increment::Minor);

        // Act
        let sorted = it.topo_sort();

        // Assert
        assert_that!(actual).is_ok();
        assert_that!(actual.unwrap().minor).is_equal_to(2);
        assert_eq!(vec!["solp", "solv"], sorted);

        let it = VersionIter::open(&conf).unwrap();
        let versions: Vec<String> = it
            .map(|v| v.places)
            .flatten()
            .map(|p| {
                return match p {
                    Place::Package(s) => s,
                    Place::Dependency(_, s) => s,
                };
            })
            .collect();
        assert_eq!(vec!["0.2.0", "0.2.0", "0.2.0"], versions)
    }

    #[test]
    fn update_complex_workspace_test() {
        // Arrange
        const W: &str = r#"
[workspace]
members = [ "a", "b", "c", "d" ]
"#;
        const A: &str = r#"
[package]
name = "a"
version = "0.1.0"
workspace = ".."

[dependencies]
x = "^0.8"
        "#;

        const B: &str = r#"
[package]
name = "b"
version = "0.1.0"
workspace = ".."

[dependencies]
x = "^0.8"
d = { path = "../d/", version = "0.1.0" }
        "#;

        const C: &str = r#"
[package]
name = "c"
version = "0.1.0"
workspace = ".."

[dependencies]
x = "^0.8"
b = { path = "../b/", version = "0.1.0" }
        "#;

        const D: &str = r#"
[package]
name = "d"
version = "0.1.0"
workspace = ".."

[dependencies]
x = "^0.8"
a = { path = "../a/", version = "0.1.0" }
        "#;

        let root_path = PathBuf::from("/");
        let fs = MemoryFS::new();
        fs.create_dir(root_path.to_str().unwrap()).unwrap();
        fs.create_dir("/a").unwrap();
        fs.create_dir("/b").unwrap();
        fs.create_dir("/c").unwrap();
        fs.create_dir("/d").unwrap();
        let root_conf = root_path.join(CARGO_CONFIG);
        let root_conf = root_conf.to_str().unwrap();
        fs.create_file(root_conf)
            .unwrap()
            .write_all(W.as_bytes())
            .unwrap();

        let ch_fn = |c: &str, d: &str| {
            let ch_conf = root_path.join(c).join(CARGO_CONFIG);
            fs.create_file(ch_conf.to_str().unwrap())
                .unwrap()
                .write_all(d.as_bytes())
                .unwrap();
        };

        ch_fn("a", A);
        ch_fn("b", B);
        ch_fn("c", C);
        ch_fn("d", D);

        let root: VfsPath = fs.into();
        let conf = root.join(CARGO_CONFIG).unwrap();
        let mut it = VersionIter::open(&conf).unwrap();

        // Act
        let result = update_configs(&conf, &mut it, Increment::Minor);

        // Assert
        assert!(result.is_ok());
        assert_eq!("0.2.0", result.unwrap().to_string());
        assert_eq!(4, it.graph.node_count());
        assert_eq!(3, it.graph.edge_count());
        let sorted = it.topo_sort();
        assert_eq!(vec!["a", "d", "b", "c"], sorted);
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

    fn new_file_system() -> MemoryFS {
        let root_path = PathBuf::from("/");
        let fs = MemoryFS::new();
        fs.create_dir(root_path.to_str().unwrap()).unwrap();
        fs.create_dir("/solv").unwrap();
        fs.create_dir("/solp").unwrap();
        let root_conf = root_path.join(CARGO_CONFIG);
        let root_conf = root_conf.to_str().unwrap();
        fs.create_file(root_conf)
            .unwrap()
            .write_all(WKS.as_bytes())
            .unwrap();

        let ch_fn = |c: &str, d: &str| {
            let ch_conf = root_path.join(c).join(CARGO_CONFIG);
            fs.create_file(ch_conf.to_str().unwrap())
                .unwrap()
                .write_all(d.as_bytes())
                .unwrap();
        };

        ch_fn("solv", SOLV);
        ch_fn("solp", SOLP);

        fs
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
