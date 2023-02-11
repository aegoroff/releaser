use crate::{new_cargo_config_path, pkg, CrateConfig};
use serde::Serialize;
use vfs::VfsPath;

#[derive(Serialize, Default)]
pub struct Scoop {
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    pub version: String,
    pub license: String,
    pub architecture: Architecture,
}

#[derive(Serialize, Default)]
pub struct Architecture {
    #[serde(rename(serialize = "64bit"))]
    pub x64: Binary,
}

#[derive(Serialize, Default)]
pub struct Binary {
    pub url: String,
    pub hash: Option<String>,
    pub bin: Vec<String>,
}

#[must_use] pub fn new_scoop(
    crate_path: VfsPath,
    binary_path: VfsPath,
    executable_name: &str,
    base_uri: &str,
) -> Option<String> {
    let crate_conf = new_cargo_config_path(&crate_path)?;
    let config = CrateConfig::open(&crate_conf).ok()?;
    let binary = pkg::new_binary_pkg(&binary_path, base_uri)?;
    let x64pkg = Binary {
        url: binary.url,
        hash: Some(binary.hash),
        bin: vec![executable_name.to_string()],
    };

    let scoop = Scoop {
        description: config.package.description.unwrap_or_default(),
        homepage: config.package.homepage,
        version: config.package.version,
        license: config.package.license.unwrap_or_default(),
        architecture: Architecture { x64: x64pkg },
    };
    let result = serde_json::to_string_pretty(&scoop).ok()?;
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CARGO_CONFIG;
    use rstest::{fixture, rstest};
    use vfs::MemoryFS;

    #[rstest]
    fn new_scoop_all_correct(root: VfsPath) {
        // Arrange
        let binary_path = root.join("x64").unwrap();

        // Act
        let result = new_scoop(root, binary_path, "solv.exe", "http://localhost");

        // Assert
        assert!(result.is_some());
        assert_eq!(
            result.unwrap().as_str(),
            r###"{
  "description": "Microsoft Visual Studio solution parsing library",
  "homepage": "https://github.com/aegoroff/solv",
  "version": "0.1.13",
  "license": "MIT",
  "architecture": {
    "64bit": {
      "url": "http://localhost/solv.tar.gz",
      "hash": "a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3",
      "bin": [
        "solv.exe"
      ]
    }
  }
}"###,
        )
    }

    #[rstest]
    fn new_scoop_binary_path_not_exist(root: VfsPath) {
        // Arrange
        let binary_path = root.join("x86").unwrap();

        // Act
        let result = new_scoop(root, binary_path, "solv.exe", "http://localhost");

        // Assert
        assert!(result.is_none());
    }

    #[rstest]
    fn new_scoop_invalid_cargo_toml() {
        // Arrange
        let root = VfsPath::new(MemoryFS::new());

        root.join("x64").unwrap().create_dir().unwrap();
        root.join(CARGO_CONFIG)
            .unwrap()
            .create_file()
            .unwrap()
            .write_all("test".as_bytes())
            .unwrap();

        let binary_path = root.join("x64").unwrap();

        binary_path
            .join("solv.tar.gz")
            .unwrap()
            .create_file()
            .unwrap()
            .write_all("123".as_bytes())
            .unwrap();

        // Act
        let result = new_scoop(root, binary_path, "solv.exe", "http://localhost");

        // Assert
        assert!(result.is_none());
    }

    #[fixture]
    fn root() -> VfsPath {
        let root = VfsPath::new(MemoryFS::new());

        root.join("x64").unwrap().create_dir().unwrap();
        root.join(CARGO_CONFIG)
            .unwrap()
            .create_file()
            .unwrap()
            .write_all(CONFIG.as_bytes())
            .unwrap();

        root.join("x64")
            .unwrap()
            .join("solv.tar.gz")
            .unwrap()
            .create_file()
            .unwrap()
            .write_all("123".as_bytes())
            .unwrap();

        root
    }

    const CONFIG: &str = r#"
[package]
name = "solp"
description = "Microsoft Visual Studio solution parsing library"
repository = "https://github.com/aegoroff/solv"
homepage = "https://github.com/aegoroff/solv"
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
}
