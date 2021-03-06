use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

extern crate semver;
extern crate serde;
extern crate toml;

const CARGO_CONFIG: &str = "Cargo.toml";

fn read_workspace(path: &str) {
    let wks_path = PathBuf::from(&path).join(CARGO_CONFIG);
    match fs::read_to_string(wks_path.to_str().unwrap()) {
        Ok(wc) => {
            let wks: WorkspaceConfig = toml::from_str(&wc).unwrap();
            let all_members: HashSet<&String> = wks.workspace.members.iter().collect();
            for member in &wks.workspace.members {
                let crate_path = PathBuf::from(&path).join(member).join(CARGO_CONFIG);
                let crate_path = crate_path.to_str().unwrap();
                match fs::read_to_string(crate_path) {
                    Ok(cc) => {
                        let crt: CrateConfig = toml::from_str(&cc).unwrap();
                        println!("crate: {} ver {}", &member, crt.package.version);
                        crt.dependencies
                            .iter()
                            .filter(|(n, _)| all_members.contains(n))
                            .map(|(n, v)| {
                                if let Dependency::Object(m) = v {
                                    if let Dependency::Plain(s) = m.get("version").unwrap() {
                                        println!("  depends on: {} ver: {}", n, s);
                                    }
                                }
                            })
                            .count();
                    }
                    Err(e) => eprintln!("{} - {}", crate_path, e),
                }
            }
        }
        Err(e) => eprintln!("{} - {}", path, e),
    }
}

#[derive(Deserialize)]
struct WorkspaceConfig {
    workspace: Workspace,
}

#[derive(Deserialize)]
struct Workspace {
    members: Vec<String>,
}

#[derive(Deserialize)]
struct CrateConfig {
    package: Package,
    dependencies: HashMap<String, Dependency>,
}

#[derive(Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_workspace_test() {
        read_workspace("/Users/egr/RustProjects/solv");
    }

    #[test]
    fn toml_parse_workspace() {
        // Arrange
        let t = r#"
[workspace]

members = [
    "solv",
    "solp",
]
        "#;

        // Act
        let cfg: WorkspaceConfig = toml::from_str(t).unwrap();

        // Assert
        assert_eq!(2, cfg.workspace.members.len());
    }

    #[test]
    fn toml_parse_crate() {
        // Arrange
        let t = r#"
[package]
name = "solv"
description = "Microsoft Visual Studio solution validator"
repository = "https://github.com/aegoroff/solv"
version = "0.1.13"
authors = ["egoroff <egoroff@gmail.com>"]
edition = "2018"
license = "MIT"
workspace = ".."

[dependencies]
prettytable-rs = "^0.8"
ansi_term = "0.12"
humantime = "2.1"
clap = "2"
fnv = "1"
solp = { path = "../solp/", version = "0.1.13" }

        "#;

        // Act
        let cfg: CrateConfig = toml::from_str(t).unwrap();

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
}
