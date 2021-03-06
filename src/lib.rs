use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use vfs::{FileSystem};

extern crate semver;
extern crate serde;
extern crate toml;
extern crate vfs;

const CARGO_CONFIG: &str = "Cargo.toml";

fn read_workspace<F: FileSystem>(path: &str, fs: F) {
    let wks_path = PathBuf::from(&path).join(CARGO_CONFIG);

    let mut wks_file = fs.open_file(wks_path.to_str().unwrap()).unwrap();
    let mut wc = String::new();
    wks_file.read_to_string(&mut wc);

    let wks: WorkspaceConfig = toml::from_str(&wc).unwrap();
    let all_members: HashSet<&String> = wks.workspace.members.iter().collect();
    for member in &wks.workspace.members {
        let crate_path = PathBuf::from(&path).join(member).join(CARGO_CONFIG);
        let crate_path = crate_path.to_str().unwrap();

        let mut crt_file = fs.open_file(crate_path).unwrap();
        let mut cc = String::new();
        crt_file.read_to_string(&mut cc);

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
    use vfs::{MemoryFS};

    #[test]
    fn read_workspace_test() {
        // Arrange
        let root_path = PathBuf::from("/");
        let fs = MemoryFS::new();
        fs.create_dir(root_path.to_str().unwrap());
        fs.create_dir("/solv");
        fs.create_dir("/solp");
        let root_conf = root_path.join(CARGO_CONFIG);
        let root_conf = root_conf.to_str().unwrap();
        fs.create_file(root_conf).unwrap().write_all(WKS.as_bytes());

        let ch1_conf = root_path.join("solv").join(CARGO_CONFIG);
        fs.create_file(ch1_conf.to_str().unwrap()).unwrap().write_all(SOLV.as_bytes());

        let ch1_conf = root_path.join("solp").join(CARGO_CONFIG);
        fs.create_file(ch1_conf.to_str().unwrap()).unwrap().write_all(SOLP.as_bytes());

        // Act
        read_workspace("/", fs);
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
