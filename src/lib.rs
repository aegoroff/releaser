mod cargo;
mod git;
pub mod workflow;

use petgraph::graphmap::DiGraphMap;
use petgraph::visit::Topo;
use semver::Version;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use toml_edit::{value, Document};
use vfs::FileSystem;

extern crate petgraph;
extern crate semver;
extern crate serde;
extern crate toml;
extern crate toml_edit;
extern crate vfs;

pub type AnyError = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, AnyError>;

const CARGO_CONFIG: &str = "Cargo.toml";
const VERSION: &str = "version";
const PACK: &str = "package";
const DEPS: &str = "dependencies";

pub struct VersionIter<'a, F: FileSystem> {
    search: HashMap<String, usize>,
    members: Vec<String>,
    root: PathBuf,
    fs: &'a F,
    graph: DiGraphMap<usize, i32>,
}

impl<'a, F: FileSystem> VersionIter<'a, F> {
    pub fn open(path: &str, fs: &'a F) -> Result<Self> {
        let root = PathBuf::from(&path);
        let wks_path = root.join(CARGO_CONFIG);

        let mut wks_file = fs.open_file(wks_path.to_str().unwrap())?;
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
            root,
            fs,
            graph,
        })
    }

    pub fn topo_sort(&self) -> Vec<String> {
        let mut topo = Topo::new(&self.graph);

        let reverted = self
            .search
            .iter()
            .map(|(k, v)| (*v, k))
            .collect::<HashMap<usize, &String>>();
        let mut result = vec![];
        loop {
            match topo.next(&self.graph) {
                Some(n) => {
                    let s = reverted.get(&n).unwrap();
                    result.push((*s).clone());
                }
                None => break,
            }
        }
        result
    }
}

impl<'a, F: FileSystem> Iterator for VersionIter<'a, F> {
    type Item = CrateVersion;

    fn next(&mut self) -> Option<Self::Item> {
        let member = self.members.pop()?;
        let crate_path = self.root.join(member).join(CARGO_CONFIG);
        let crate_path = crate_path.to_str()?;

        let conf = CrateConfig::open(self.fs, crate_path).unwrap_or_default();
        if conf.package.version.is_empty() {
            return None;
        }

        let mut item = conf.new_version(crate_path);

        let deps = conf
            .dependencies
            .iter()
            .filter(|(n, _)| self.search.contains_key(*n))
            .filter_map(|(n, v)| {
                if let Dependency::Object(m) = v {
                    let d = m.get(VERSION)?;
                    if let Dependency::Plain(s) = d {
                        return Some(Place::Dependency(n.clone(), s.clone()));
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

pub fn update_configs<F, I>(fs: &F, iter: &mut I, incr: Increment) -> Result<Version>
where
    F: FileSystem,
    I: Iterator<Item = CrateVersion>,
{
    let result = Version::parse("0.0.0")?;

    let result = iter
        .by_ref()
        .map(|config| update_config(fs, &config, incr))
        .filter(|v| v.is_ok())
        .map(|r| r.unwrap())
        .fold(result, |r, v| r.max(v));

    Ok(result)
}

fn update_config<F>(fs: &F, version: &CrateVersion, incr: Increment) -> Result<Version>
where
    F: FileSystem,
{
    let mut file = fs.open_file(&version.path)?;
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

    let mut f = fs.create_file(&version.path)?;
    let changed = doc.to_string_in_original_order();
    f.write_all(changed.as_bytes())?;
    Ok(result)
}

fn increment(v: &String, i: Increment) -> Result<Version> {
    let mut v = Version::parse(v)?;
    match i {
        Increment::Major => v.increment_major(),
        Increment::Minor => v.increment_minor(),
        Increment::Patch => v.increment_patch(),
    }
    Ok(v)
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
    pub fn open<F: FileSystem>(fs: &F, path: &str) -> Result<Self> {
        let mut file = fs.open_file(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let conf: CrateConfig = toml::from_str(&content).unwrap();
        Ok(conf)
    }

    pub fn new_version(&self, path: &str) -> CrateVersion {
        let places = vec![Place::Package(self.package.version.clone())];

        CrateVersion {
            path: String::from(path),
            places,
        }
    }
}

#[derive(Deserialize, Default)]
struct Package {
    name: String,
    version: String,
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

#[derive(Copy, Clone)]
pub enum Increment {
    Major,
    Minor,
    Patch,
}

#[cfg(test)]
mod tests {
    use super::*;
    use vfs::MemoryFS;

    #[test]
    fn read_workspace_test() {
        // Arrange
        let fs = new_file_system();
        let it = VersionIter::open("/", &fs).unwrap();

        // Act
        let versions = it.count();

        // Assert
        assert_eq!(2, versions);
    }

    #[test]
    fn read_empty_workspace_test() {
        // Arrange
        let root_path = PathBuf::from("/");
        let fs = MemoryFS::new();
        fs.create_dir(root_path.to_str().unwrap()).unwrap();

        // Act
        let result = VersionIter::open("/", &fs);

        // Assert
        assert!(result.is_err())
    }

    #[test]
    fn update_workspace_patch_test() {
        // Arrange
        let fs = new_file_system();
        let mut it = VersionIter::open("/", &fs).unwrap();

        // Act
        let result = update_configs(&fs, &mut it, Increment::Patch);

        // Assert
        assert!(result.is_ok());
        assert_eq!("0.1.14", result.unwrap().to_string());
        assert_updated_files(&fs, "0.1.14");
        assert_eq!(2, it.graph.node_count());
        assert_eq!(1, it.graph.edge_count());
        let sorted = it.topo_sort();
        assert_eq!(vec!["solp", "solv"], sorted);
    }

    #[test]
    fn update_workspace_minor_test() {
        // Arrange
        let fs = new_file_system();
        let mut it = VersionIter::open("/", &fs).unwrap();

        // Act
        let result = update_configs(&fs, &mut it, Increment::Minor);

        // Assert
        assert!(result.is_ok());
        assert_eq!("0.2.0", result.unwrap().to_string());
        assert_updated_files(&fs, "0.2.0");
        assert_eq!(2, it.graph.node_count());
        assert_eq!(1, it.graph.edge_count());
        let sorted = it.topo_sort();
        assert_eq!(vec!["solp", "solv"], sorted);
    }

    #[test]
    fn update_workspace_major_test() {
        // Arrange
        let fs = new_file_system();
        let mut it = VersionIter::open("/", &fs).unwrap();

        // Act
        let result = update_configs(&fs, &mut it, Increment::Major);

        // Assert
        assert!(result.is_ok());
        assert_eq!("1.0.0", result.unwrap().to_string());
        assert_updated_files(&fs, "1.0.0");
        assert_eq!(2, it.graph.node_count());
        assert_eq!(1, it.graph.edge_count());
        let sorted = it.topo_sort();
        assert_eq!(vec!["solp", "solv"], sorted);
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

    fn assert_updated_files(fs: &MemoryFS, ver: &str) {
        let it = VersionIter::open("/", fs).unwrap();
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
        assert_eq!(vec![ver, ver, ver], versions)
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

        let ch1_conf = root_path.join("solv").join(CARGO_CONFIG);
        fs.create_file(ch1_conf.to_str().unwrap())
            .unwrap()
            .write_all(SOLV.as_bytes())
            .unwrap();

        let ch1_conf = root_path.join("solp").join(CARGO_CONFIG);
        fs.create_file(ch1_conf.to_str().unwrap())
            .unwrap()
            .write_all(SOLP.as_bytes())
            .unwrap();
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
