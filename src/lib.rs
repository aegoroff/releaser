use semver::Version;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use toml_edit::{value, Document};
use vfs::FileSystem;

extern crate semver;
extern crate serde;
extern crate toml;
extern crate toml_edit;
extern crate vfs;

pub type AnyError = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, AnyError>;

const CARGO_CONFIG: &str = "Cargo.toml";

pub struct VersionIter<'a, F: FileSystem> {
    search: HashSet<String>,
    members: Vec<String>,
    root: PathBuf,
    fs: &'a F,
}

impl<'a, F: FileSystem> VersionIter<'a, F> {
    pub fn new(path: &str, fs: &'a F) -> Result<Self> {
        let root = PathBuf::from(&path);
        let wks_path = root.join(CARGO_CONFIG);

        let mut wks_file = fs.open_file(wks_path.to_str().unwrap())?;
        let mut wc = String::new();
        wks_file.read_to_string(&mut wc)?;

        let wks: WorkspaceConfig = toml::from_str(&wc)?;
        let search: HashSet<String> = wks.workspace.members.iter().cloned().collect();
        let members  = wks.workspace.members;
        Ok(Self{search, members, root, fs})
    }
}

impl<'a, F: FileSystem> Iterator for VersionIter<'a, F> {
    type Item = CrateVersion;

    fn next(&mut self) -> Option<Self::Item> {
        let member = self.members.pop()?;
        let crate_path = self.root.join(member).join(CARGO_CONFIG);
        let crate_path = crate_path.to_str()?;

        if let Ok(mut crt_file) = self.fs.open_file(crate_path) {
            let mut cc = String::new();
            let ok = crt_file.read_to_string(&mut cc).is_ok();
            if !ok {
                return None;
            }
            let crt: CrateConfig = toml::from_str(&cc).unwrap();

            let mut places = vec![Place::Package(crt.package.version)];

            let deps = crt
                .dependencies
                .iter()
                .filter(|(n, _)| self.search.contains(*n))
                .filter_map(|(n, v)| {
                    if let Dependency::Object(m) = v {
                        if let Some(d) = m.get("version") {
                            if let Dependency::Plain(s) = d {
                                return Some(Place::Dependency(n.clone(), s.clone()));
                            }
                        }
                    }
                    None
                });

            places.extend(deps);
            return  Some(CrateVersion {
                path: String::from(crate_path),
                places,
            });
        }

        None
    }
}

pub fn update_configs<F: FileSystem>(fs: &F, iter: VersionIter<F>) -> Result<()> {
    for config in iter {
        let mut file = fs.open_file(&config.path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let mut doc = content.parse::<Document>()?;

        for place in &config.places {
            match place {
                Place::Package(ver) => {
                    let mut v = Version::parse(ver)?;
                    v.increment_patch();
                    doc["package"]["version"] = value(v.to_string());
                }
                Place::Dependency(n, ver) => {
                    let mut v = Version::parse(ver)?;
                    v.increment_patch();
                    doc["dependencies"][n]["version"] = value(v.to_string());
                }
            }
        }

        let mut f = fs.create_file(&config.path)?;
        let changed = doc.to_string_in_original_order();
        f.write_all(changed.as_bytes())?;
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use vfs::MemoryFS;

    #[test]
    fn read_workspace_test() {
        // Arrange
        let fs = new_file_system();
        let it = VersionIter::new("/", &fs).unwrap();

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
        let result = VersionIter::new("/", &fs);

        // Assert
        assert!(result.is_err())
    }

    #[test]
    fn update_workspace_test() {
        // Arrange
        let fs = new_file_system();
        let it = VersionIter::new("/", &fs).unwrap();

        // Act
        let result = update_configs(&fs, it);

        // Assert
        assert!(result.is_ok());
        let it = VersionIter::new("/", &fs).unwrap();
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
        assert_eq!(vec!["0.1.14", "0.1.14", "0.1.14"], versions)
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
            assert!(o.contains_key("version"));
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
