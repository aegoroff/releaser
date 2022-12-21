use crate::{CrateConfig, CrateVersion, Dependency, Place, WorkspaceConfig, CARGO_CONFIG, VERSION};
use crate::error::FileError;
use petgraph::algo::DfsSpace;
use petgraph::graphmap::DiGraphMap;
use std::collections::HashMap;
use std::io::Read;
use vfs::VfsPath;

pub struct VersionIter<'a> {
    search: HashMap<String, usize>,
    members: Vec<String>,
    workspace_config_path: &'a VfsPath,
    graph: DiGraphMap<usize, i32>,
}

impl<'a> VersionIter<'a> {
    pub fn open(path: &'a VfsPath) -> crate::Result<Self> {
        let mut wks_file = match path.open_file() {
            Ok(it) => it,
            Err(err) => return Err(Box::new(FileError::from(err))),
        };
        let mut wc = String::new();
        wks_file.read_to_string(&mut wc)?;
        let wks: WorkspaceConfig = toml::from_str(&wc)?;
        let search: HashMap<String, usize> = wks
            .workspace
            .members
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, s)| (s, i))
            .collect();
        let members = wks.workspace.members;

        let graph = DiGraphMap::new();
        Ok(Self {
            search,
            members,
            workspace_config_path: path,
            graph,
        })
    }

    pub fn topo_sort(&self) -> Vec<String> {
        let reverted = self
            .search
            .iter()
            .map(|(k, v)| (*v, k))
            .collect::<HashMap<usize, &String>>();

        let mut space = DfsSpace::new(&self.graph);
        let sorted = petgraph::algo::toposort(&self.graph, Some(&mut space)).unwrap_or_default();

        sorted
            .into_iter()
            .map(|g| *reverted.get(&g).unwrap())
            .cloned()
            .collect()
    }
}

impl<'a> Iterator for VersionIter<'a> {
    type Item = CrateVersion;

    fn next(&mut self) -> Option<Self::Item> {
        let member = self.members.pop()?;
        let root = self.workspace_config_path.parent();
        let config_path = crate::join(&root, &member)?;
        let config_path = crate::join(&config_path, CARGO_CONFIG)?;

        let conf = CrateConfig::open(&config_path).ok()?;

        let mut item = conf.new_version(member);

        let deps = conf
            .dependencies?
            .into_iter()
            .filter(|(n, _)| self.search.contains_key(n))
            .filter_map(|(n, v)| {
                if let Dependency::Object(m) = v {
                    let d = m.get(VERSION)?;
                    if let Dependency::Plain(s) = d {
                        return Some(Place::Dependency(n, s.clone()));
                    }
                }
                None
            });

        item.places.extend(deps);

        let to = self.search.get(&conf.package.name)?;

        for place in item.places.iter() {
            if let Place::Dependency(n, _) = place {
                let from = self.search.get(n)?;
                self.graph.add_edge(*from, *to, -1);
            }
        }

        Some(item)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use vfs::{FileSystem, MemoryFS};

    use super::*;
    use crate::version_iter::VersionIter;
    use crate::{update_configs, Increment};
    use rstest::*;

    #[test]
    fn read_empty_workspace_test() {
        // Arrange
        let root_path = PathBuf::from("/");
        let fs = MemoryFS::new();
        fs.create_dir(root_path.to_str().unwrap()).unwrap();
        let root: VfsPath = fs.into();
        let conf = root.join(CARGO_CONFIG).unwrap();

        // Act
        let result = VersionIter::open(&conf);

        // Assert
        assert!(result.is_err());
    }

    #[rstest]
    fn read_workspace_test(root: VfsPath) {
        // Arrange
        let conf = root.join(CARGO_CONFIG).unwrap();
        let it = VersionIter::open(&conf).unwrap();

        // Act
        let versions = it.count();

        // Assert
        assert_eq!(versions, 2);
    }

    #[rstest]
    #[case::patch(Increment::Patch, "0.1.14")]
    #[case::minor(Increment::Minor, "0.2.0")]
    #[case::major(Increment::Major, "1.0.0")]
    #[trace]
    fn update_workspace_version_change_tests(
        root: VfsPath,
        #[case] incr: Increment,
        #[case] expected: String,
    ) {
        // Arrange
        let conf = root.join(CARGO_CONFIG).unwrap();
        let mut it = VersionIter::open(&conf).unwrap();

        // Act
        let actual = update_configs(&conf, &mut it, incr).unwrap().to_string();

        // Assert
        assert_eq!(actual, expected);
    }

    #[rstest]
    fn version_iter_topo_sort_test(root: VfsPath) {
        // Arrange
        let conf = root.join(CARGO_CONFIG).unwrap();
        let mut it = VersionIter::open(&conf).unwrap();
        let actual = update_configs(&conf, &mut it, Increment::Minor);

        // Act
        let sorted = it.topo_sort();

        // Assert
        assert!(actual.is_ok());
        assert_eq!(actual.unwrap().minor, 2);
        assert_eq!(vec!["solp", "solv"], sorted);

        let it = VersionIter::open(&conf).unwrap();
        let versions: Vec<String> = it
            .map(|v| v.places)
            .flatten()
            .map(|p| {
                return match p {
                    Place::Package(s) => s,
                    Place::Dependency(_, s) => s,
                };
            })
            .collect();
        assert_eq!(vec!["0.2.0", "0.2.0", "0.2.0"], versions)
    }

    #[test]
    fn update_complex_workspace_test() {
        // Arrange
        const W: &str = r#"
[workspace]
members = [ "a", "b", "c", "d" ]
"#;
        const A: &str = r#"
[package]
name = "a"
version = "0.1.0"
workspace = ".."
        "#;

        const B: &str = r#"
[package]
name = "b"
version = "0.1.0"
workspace = ".."

[dependencies]
x = "^0.8"
d = { path = "../d/", version = "0.1.0" }
        "#;

        const C: &str = r#"
[package]
name = "c"
version = "0.1.0"
workspace = ".."

[dependencies]
x = "^0.8"
b = { path = "../b/", version = "0.1.0" }
        "#;

        const D: &str = r#"
[package]
name = "d"
version = "0.1.0"
workspace = ".."

[dependencies]
x = "^0.8"
a = { path = "../a/", version = "0.1.0" }
        "#;

        let root = VfsPath::new(MemoryFS::new());
        root.join("a").unwrap().create_dir().unwrap();
        root.join("b").unwrap().create_dir().unwrap();
        root.join("c").unwrap().create_dir().unwrap();
        root.join("d").unwrap().create_dir().unwrap();
        let root_conf = root.join(CARGO_CONFIG).unwrap();
        root_conf
            .create_file()
            .unwrap()
            .write_all(W.as_bytes())
            .unwrap();

        let ch_fn = |c: &str, d: &str| {
            let ch_conf = root.join(c).unwrap().join(CARGO_CONFIG).unwrap();
            ch_conf
                .create_file()
                .unwrap()
                .write_all(d.as_bytes())
                .unwrap();
        };

        ch_fn("a", A);
        ch_fn("b", B);
        ch_fn("c", C);
        ch_fn("d", D);

        let conf = root.join(CARGO_CONFIG).unwrap();
        let mut it = VersionIter::open(&conf).unwrap();

        // Act
        let result = update_configs(&conf, &mut it, Increment::Minor);

        // Assert
        assert!(result.is_ok());
        assert_eq!("0.2.0", result.unwrap().to_string());
        assert_eq!(4, it.graph.node_count());
        assert_eq!(3, it.graph.edge_count());
        let sorted = it.topo_sort();
        assert_eq!(vec!["a", "d", "b", "c"], sorted);
    }

    #[fixture]
    fn root() -> VfsPath {
        let root = VfsPath::new(MemoryFS::new());

        root.join("solv").unwrap().create_dir().unwrap();
        root.join("solp").unwrap().create_dir().unwrap();
        root.join(CARGO_CONFIG)
            .unwrap()
            .create_file()
            .unwrap()
            .write_all(WKS.as_bytes())
            .unwrap();

        let ch_fn = |c: &str, d: &str| {
            let ch_conf = root.join(c).unwrap().join(CARGO_CONFIG).unwrap();
            ch_conf
                .create_file()
                .unwrap()
                .write_all(d.as_bytes())
                .unwrap();
        };

        ch_fn("solv", SOLV);
        ch_fn("solp", SOLP);

        root
    }

    const WKS: &str = r#"
[workspace]

members = [
    "solv",
    "solp",
]
        "#;

    const SOLP: &str = r#"
[package]
name = "solp"
description = "Microsoft Visual Studio solution parsing library"
repository = "https://github.com/aegoroff/solv"
version = "0.1.13"
authors = ["egoroff <egoroff@gmail.com>"]
edition = "2018"
license = "MIT"
workspace = ".."

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[build-dependencies] # <-- We added this and everything after!
lalrpop = "0.19"

[dependencies]
lalrpop-util = "0.19"
regex = "1"
jwalk = "0.6"
phf = { version = "0.8", features = ["macros"] }
itertools = "0.10"

        "#;

    const SOLV: &str = r#"
[package]
name = "solv"
description = "Microsoft Visual Studio solution validator"
repository = "https://github.com/aegoroff/solv"
version = "0.1.13"
authors = ["egoroff <egoroff@gmail.com>"]
edition = "2018"
license = "MIT"
workspace = ".."

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
prettytable-rs = "^0.8"
ansi_term = "0.12"
humantime = "2.1"
clap = "2"
fnv = "1"
solp = { path = "../solp/", version = "0.1.13" }

        "#;
}
