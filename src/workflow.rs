use crate::git;
use crate::cargo;
use crate::{Increment, VersionIter};
use std::path::PathBuf;
use vfs::PhysicalFS;
use std::{thread, time};

pub fn release(path: &str, incr: Increment) -> crate::Result<()> {
    let root_path = PathBuf::from(path);
    let root = PhysicalFS::new(root_path);

    let mut it = VersionIter::open("/", &root)?;
    let version = crate::update_configs(&root, &mut it, incr)?;

    let ver = format!("v{}", version);
    let commit_msg = format!("New release {}", &ver);
    git::commit(&commit_msg, path)?;

    let minute = time::Duration::from_secs(60);
    let crates_to_publish = it.topo_sort();
    for (i, publish) in crates_to_publish.iter().enumerate() {
        cargo::publish(path, publish)?;
        if i < crates_to_publish.len() - 1 {
            thread::sleep(minute);
        }
    }

    git::create_tag(path, &ver)?;
    git::push_tag(path, &ver)?;

    Ok(())
}
