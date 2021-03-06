use serde::{Deserialize};
use std::collections::{HashMap};

extern crate semver;
extern crate toml;
extern crate serde;

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

#[derive(Deserialize)]
#[serde(untagged)]
enum  Dependency {
    Plain(String),
    Object(HashMap<String, Dependency>)
}

#[cfg(test)]
mod tests {
    use super::*;

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
