use color_eyre::eyre::Result;
use sha2::{Digest, Sha256};
use std::io::{BufReader, Read};
use vfs::VfsPath;

pub fn calculate_sha256(path: &VfsPath) -> Result<String> {
    let file = path.open_file()?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();

    let mut buf = [0u8; 8192];

    loop {
        let r = reader.read(&mut buf);
        if let Ok(n) = r {
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        } else {
            break;
        }
    }
    let result = &hasher.finalize()[..];

    Ok(hex::encode(result))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_in_result)]
    #![allow(clippy::unwrap_used)]
    use super::*;
    use vfs::MemoryFS;

    #[test]
    fn calculate_sha256_test() {
        // Arrange
        let root: VfsPath = MemoryFS::new().into();
        let file_path = root.join("file.txt");
        let file_path = file_path.unwrap();
        file_path
            .create_file()
            .unwrap()
            .write_all("123".as_bytes())
            .unwrap();

        // Act
        let hash = calculate_sha256(&file_path).unwrap();

        // Assert
        assert_eq!(
            hash.as_str(),
            "a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3"
        );
    }

    #[test]
    fn calculate_sha256_file_not_exist_test() {
        // Arrange
        let root: VfsPath = MemoryFS::new().into();
        let file_path = root.join("file.txt");
        let file_path = file_path.unwrap();

        // Act
        let hash = calculate_sha256(&file_path);

        // Assert
        assert!(hash.is_err());
    }
}
