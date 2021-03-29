[![crates.io](https://img.shields.io/crates/v/releaser.svg)](https://crates.io/crates/releaser)

# Rust workspace release procedure
1. Read existing version from crate's toml file
2. Increment version in all workspace's crate's toml files and it's dependencies
3. Commit all version changes
4. Create new git tag
5. Run **cargo publish --manifest-path …**
6. Wait some time (20 seconds by default) before publish next crate so as to use new version   
7. Push git tag

# Rust crate release procedure
1. Read existing version from crate's toml file
2. Increment version in the toml file
3. Commit all version changes
4. Create new git tag
5. Run **cargo publish --manifest-path …**
6. Push git tag

Command line syntax:
--------------------
```
Rust releasing workspace tool

USAGE:
    releaser [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    c       Release single crate specified by path
    help    Prints this message or the help of the given subcommand(s)
    w       Release workspace specified by path
```
Releasing workspace
```
Release workspace specified by path

USAGE:
    releaser w [OPTIONS] <INCR> <PATH>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --delay <delay>    Delay in seconds between publish next workflow's crate [default: 20]

ARGS:
    <INCR>    Version increment. One of the following: major, minor or patch
    <PATH>    Sets workspace root path
```
Releasing simple crate
```
Release single crate specified by path

USAGE:
    releaser c <INCR> <PATH>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <INCR>    Version increment. One of the following: major, minor or patch
    <PATH>    Sets crate's root path
```