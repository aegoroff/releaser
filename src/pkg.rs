use std::path::PathBuf;
use std::fs;
use vfs::PhysicalFS;
use crate::hash;
use crate::resource::Resource;
use serde::Serialize;

#[derive(Serialize, Default)]
pub struct Package {
    pub url: String,
    pub hash: String,
}

pub fn new_binary_pkg(path: &str, base_uri: &str) -> Option<Package> {
    let sha256 = calculate_sha256(path);
    let mut resource = Resource::new(base_uri)?;
    sha256.map(|(h, f)| {
        let file = f.file_name().unwrap().to_str().unwrap();
        resource.append_path(file);
        Package {
            url: resource.to_string(),
            hash: h,
        }
    })
}

fn calculate_sha256(dir: &str) -> Option<(String, PathBuf)> {
    let dir_content = fs::read_dir(dir);
    if let Ok(d) = dir_content {
        let file = d
            .filter(|f| f.is_ok())
            .map(|x| x.unwrap())
            .map(|x| x.path())
            .find(|x| x.extension().is_some() && x.extension().unwrap().eq("gz"))
            .unwrap_or_default();

        let root_path = PathBuf::from(dir);
        let fs = PhysicalFS::new(root_path);
        let hash =
            hash::calculate_sha256(file.file_name().unwrap_or_default().to_str().unwrap(), &fs)
                .unwrap_or_default();
        Some((hash, file))
    } else {
        None
    }
}
