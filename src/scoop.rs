use crate::workflow::Crate;
use crate::{pkg, CrateConfig};
use serde::Serialize;
use std::path::PathBuf;
use vfs::{PhysicalFS, VfsPath};

#[derive(Serialize, Default)]
pub struct Scoop {
    pub description: String,
    #[serde(rename(serialize = "64bit"), skip_serializing_if = "Option::is_none")]
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

pub fn new_scoop(
    crate_path: &str,
    binary_path: &str,
    executable_name: &str,
    base_uri: &str,
) -> Option<String> {
    let crate_root: VfsPath = PhysicalFS::new(PathBuf::from(crate_path)).into();
    let crate_conf = Crate::open(&crate_root).unwrap();
    let config = CrateConfig::open(&crate_conf);

    let bin_root = PhysicalFS::new(PathBuf::from(binary_path)).into();

    let binary = pkg::new_binary_pkg(&bin_root, base_uri);
    let x64pkg: Binary;
    match binary {
        None => return None,
        Some(p) => {
            x64pkg = Binary {
                url: p.url,
                hash: Some(p.hash),
                bin: vec![executable_name.to_string()],
            }
        }
    }

    if let Ok(c) = config {
        let scoop = Scoop {
            description: c.package.description.unwrap_or_default(),
            homepage: c.package.homepage,
            version: c.package.version,
            license: c.package.license.unwrap_or_default(),
            architecture: Architecture { x64: x64pkg },
        };
        let result = serde_json::to_string_pretty(&scoop);
        match result {
            Ok(r) => Some(r),
            Err(_) => None,
        }
    } else {
        None
    }
}
