[![crates.io](https://img.shields.io/crates/v/releaser.svg)](https://crates.io/crates/releaser)
[![Rust](https://github.com/aegoroff/releaser/actions/workflows/rust.yml/badge.svg)](https://github.com/aegoroff/releaser/actions/workflows/rust.yml)

# Installation
Install Rust and then run:
```
cargo install releaser
```
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
    releaser.exe [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    b       Create brew package manager Formula (package definition file) to publish it into a tap (MacOS and Linux
            only)
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
Creating brew package manager Formula (package definition file) to publish it into a tap (MacOS and Linux
only)
```
Create brew package manager Formula (package definition file) to publish it into a tap (MacOS and Linux only)

USAGE:
    releaser.exe b [OPTIONS] --base <base> --crate <crate>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --base <base>        Base URI of downloaded artifacts
    -c, --crate <crate>      Sets crate's path to get formula's data
    -l, --linux <linux>      Sets linux package path
    -m, --macos <macos>      Sets Mac OS package path
    -u, --output <output>    File path to save result to. If not set result will be written into stdout
```