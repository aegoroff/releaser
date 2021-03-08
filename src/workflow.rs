use crate::cargo;
use crate::git;
use crate::{Increment, VersionIter};
use std::path::PathBuf;
use std::{thread, time};
use vfs::PhysicalFS;

pub fn release_workspace(path: &str, incr: Increment) -> crate::Result<()> {
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
        // delay needed between crates to avoid publish failure in case of dependencies
        // crates.io index dont updated instantly
        if i < crates_to_publish.len() - 1 {
            println!(" Waiting after publish {} ...", publish);
            thread::sleep(minute);
        }
    }

    git::create_tag(path, &ver)?;
    git::push_tag(path, &ver)?;

    Ok(())
}
