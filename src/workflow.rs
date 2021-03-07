use crate::git;
use crate::cargo;
use crate::{Increment, VersionIter};
use std::path::PathBuf;
use vfs::PhysicalFS;

pub fn release(path: &str, incr: Increment) -> crate::Result<()> {
    let root_path = PathBuf::from(path);
    let root = PhysicalFS::new(root_path);

    let mut it = VersionIter::open("/", &root)?;
    let version = crate::update_configs(&root, &mut it, incr)?;

    let ver = format!("v{}", version);
    let commit_msg = format!("New release {}", &ver);
    git::commit(&commit_msg, path)?;

    let crates_to_publish = it.topo_sort();
    for publish in &crates_to_publish {
        while cargo::publish(path, publish).is_err() {}
    }

    git::create_tag(path, &ver)?;
    git::push_tag(path, &ver)?;

    Ok(())
}
