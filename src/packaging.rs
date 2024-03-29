use crate::hash;
use crate::resource::Resource;
use color_eyre::eyre::{eyre, Result};
use serde::Serialize;
use vfs::VfsPath;

const PKG_EXTENSION: &str = "gz";

#[derive(Serialize, Default, Debug)]
pub struct Package {
    pub url: String,
    pub hash: String,
}

pub fn new_binary_pkg(path: &VfsPath, base_uri: &str) -> Result<Package> {
    let (hash, file) = calculate_sha256(path)?;
    let mut resource = Resource::new(base_uri)?;
    resource.append_path(&file);
    Ok(Package {
        url: resource.to_string(),
        hash,
    })
}

fn calculate_sha256(path: &VfsPath) -> Result<(String, String)> {
    let mut it = path.read_dir()?;
    let file_name = it.find(|x| {
        if let Some(ext) = x.extension() {
            ext.eq(PKG_EXTENSION)
        } else {
            false
        }
    });

    if let Some(file_name) = file_name {
        let hash = hash::calculate_sha256(&file_name)?;
        Ok((hash, file_name.filename()))
    } else {
        Err(eyre!("No file with extension {PKG_EXTENSION} found"))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_in_result)]
    #![allow(clippy::unwrap_used)]
    use super::*;
    use vfs::MemoryFS;

    #[test]
    fn new_binary_pkg_gz_file_exists_test() {
        // Arrange
        let root: VfsPath = MemoryFS::new().into();
        let file_path = root.join("f.tar.gz").unwrap();
        file_path
            .create_file()
            .unwrap()
            .write_all("123".as_bytes())
            .unwrap();

        // Act
        let p = new_binary_pkg(&root, "http://x").unwrap();

        // Assert
        assert_eq!(
            p.hash.as_str(),
            "a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3"
        );
        assert_eq!(p.url.as_str(), "http://x/f.tar.gz");
    }

    #[test]
    fn new_binary_pkg_gz_file_not_exists_test() {
        // Arrange
        let root: VfsPath = MemoryFS::new().into();
        let dir_path = root.join("d").unwrap();
        dir_path.create_dir().unwrap();
        root.join("d")
            .unwrap()
            .join("f.txt")
            .unwrap()
            .create_file()
            .unwrap()
            .write_all("123".as_bytes())
            .unwrap();

        // Act
        let p = new_binary_pkg(&dir_path, "http://x");

        // Assert
        assert!(p.is_err());
    }
}
