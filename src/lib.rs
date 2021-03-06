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

fn read_workspace_versions<F: FileSystem>(path: &str, fs: &F) -> Result<Vec<CrateVersion>> {
    let wks_path = PathBuf::from(&path).join(CARGO_CONFIG);

    let mut wks_file = fs.open_file(wks_path.to_str().unwrap())?;
    let mut wc = String::new();
    wks_file.read_to_string(&mut wc)?;

    let mut result = Vec::new();

    let wks: WorkspaceConfig = toml::from_str(&wc)?;
    let all_members: HashSet<&String> = wks.workspace.members.iter().collect();
    for member in all_members.iter() {
        let crate_path = PathBuf::from(&path).join(member).join(CARGO_CONFIG);
        let crate_path = crate_path.to_str().unwrap();

        let mut crt_file = fs.open_file(crate_path)?;
        let mut cc = String::new();
        crt_file.read_to_string(&mut cc)?;

        let crt: CrateConfig = toml::from_str(&cc)?;
        let v = CrateVersion {
            path: String::from(crate_path),
            place: Place::Package(crt.package.version),
        };
        result.push(v);
        let deps = crt
            .dependencies
            .iter()
            .filter(|(n, _)| all_members.contains(*n))
            .filter_map(|(n, v)| {
                if let Dependency::Object(m) = v {
                    if let Some(d) = m.get("version") {
                        if let Dependency::Plain(s) = d {
                            return Some(CrateVersion {
                                path: String::from(crate_path),
                                place: Place::Dependency(n.clone(), s.clone()),
                            });
                        }
                    }
                }
                None
            });
        result.extend(deps);
    }
    Ok(result)
}

pub fn update_configs<F: FileSystem>(configs: &Vec<CrateVersion>, fs: &F) -> Result<()> {
    for config in configs {
        let mut file = fs.open_file(&config.path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let mut doc = content.parse::<Document>().unwrap();

        match &config.place {
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

#[derive(Debug)]
pub struct CrateVersion {
    path: String,
    place: Place,
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

        // Act
        let result = read_workspace_versions("/", &fs);

        // Assert
        assert!(result.is_ok());
        assert_eq!(3, result.unwrap().len());
    }

    #[test]
    fn read_empty_workspace_test() {
        // Arrange
        let root_path = PathBuf::from("/");
        let fs = MemoryFS::new();
        fs.create_dir(root_path.to_str().unwrap()).unwrap();

        // Act
        let result = read_workspace_versions("/", &fs);

        // Assert
        assert!(result.is_err())
    }

    #[test]
    fn update_workspace_test() {
        // Arrange
        let fs = new_file_system();
        let configs = read_workspace_versions("/", &fs).unwrap();

        // Act
        let result = update_configs(&configs, &fs);

        // Assert
        assert!(result.is_ok());
        let configs = read_workspace_versions("/", &fs).unwrap();
        let versions: Vec<&String> = configs
            .iter()
            .map(|v| {
                return match &v.place {
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
