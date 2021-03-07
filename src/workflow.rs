use crate::git;
use crate::{Increment, VersionIter};
use std::path::PathBuf;
use vfs::PhysicalFS;

pub fn release(path: &str, incr: Increment) -> crate::Result<()> {
    let root_path = PathBuf::from(path);
    let root = PhysicalFS::new(root_path);

    let it = VersionIter::open("/", &root)?;
    let version = crate::update_configs(&root, it, incr)?;

    let commit_msg = format!("New release {}", version);
    let mut commit = git::commit(&commit_msg, path)?;
    commit.wait()?;

    Ok(())
}
