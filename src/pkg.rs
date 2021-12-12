use crate::hash;
use crate::resource::Resource;
use serde::Serialize;
use vfs::VfsPath;

const PKG_EXTENSION: &str = "gz";

#[derive(Serialize, Default, Debug)]
pub struct Package {
    pub url: String,
    pub hash: String,
}

pub fn new_binary_pkg(path: &VfsPath, base_uri: &str) -> Option<Package> {
    let sha256 = calculate_sha256(path);
    let mut resource = Resource::new(base_uri)?;
    sha256.map(|(h, f)| {
        resource.append_path(&f);
        Package {
            url: resource.to_string(),
            hash: h,
        }
    })
}

fn calculate_sha256(path: &VfsPath) -> Option<(String, String)> {
    let file_name = match path.read_dir() {
        Ok(it) => it
            .filter(|x| x.extension().is_some())
            .find(|x| x.extension().unwrap().eq(PKG_EXTENSION)),
        Err(_) => None,
    }?;

    let hash = hash::calculate_sha256(&file_name).unwrap_or_default();
    Some((hash, file_name.filename()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;
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
        assert_that!(p.hash.as_str())
            .is_equal_to("a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3");
        assert_that!(p.url.as_str()).is_equal_to("http://x/f.tar.gz");
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
        assert_that!(p).is_none();
    }
}
