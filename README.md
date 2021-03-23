# Rust workspace release procedure
1. Read existing version from crate's toml file
2. Increment version in all workspace's crate's toml files and it's dependencies
3. Commit all version changes
4. Create new git tag
5. Run **cargo publish --manifest-path …**
6. Wait some time (30 seconds by default) before publish next crate so as to use new version   
7. Push git tag

# Rust crate release procedure
1. Read existing version from crate's toml file
2. Increment version in the toml file
3. Commit all version changes
4. Create new git tag
5. Run **cargo publish --manifest-path …**
6. Push git tag
