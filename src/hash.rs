use std::io::{BufReader, Read};

use sha2::{Digest, Sha256};
use vfs::{FileSystem, VfsError};

pub fn calculate_sha256<F: FileSystem>(path: &str, fs: &F) -> Result<String, VfsError> {
    let file = fs.open_file(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();

    let mut buf = [0u8; 4096];

    loop {
        let r = reader.read(&mut buf);
        match r {
            Ok(n) => {
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n])
            }
            Err(_) => break,
        }
    }
    let result = &hasher.finalize()[..];

    Ok(hex::encode(result))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use vfs::MemoryFS;

    use super::*;

    #[test]
    fn calculate_sha256_test() {
        // Arrange
        let root_path = PathBuf::from("/");
        let fs = MemoryFS::new();
        fs.create_dir(root_path.to_str().unwrap()).unwrap();
        let file_path = root_path.join("file.txt");
        let file_path = file_path.to_str().unwrap();
        fs.create_file(file_path)
            .unwrap()
            .write_all("123".as_bytes())
            .unwrap();

        // Act
        let hash = calculate_sha256(file_path, &fs).unwrap();

        // Assert
        assert_eq!(
            "a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3",
            hash
        );
    }
}
