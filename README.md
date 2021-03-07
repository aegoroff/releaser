# Rust workspace release procedure
1. Read existing version from crate's toml file
2. Increment version in all workspace's crate's toml files and it's dependencies
3. Commit all version changes
4. Create new git tag
5. Run **cargo publish --manifest-path …**
6. Push git tag
