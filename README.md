[![crates.io](https://img.shields.io/crates/v/releaser.svg)](https://crates.io/crates/releaser)
[![Rust](https://github.com/aegoroff/releaser/actions/workflows/rust.yml/badge.svg)](https://github.com/aegoroff/releaser/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/aegoroff/releaser/branch/master/graph/badge.svg?token=A2vtLxosWU)](https://codecov.io/gh/aegoroff/releaser)
[![](https://tokei.rs/b1/github/aegoroff/releaser?category=code)](https://github.com/XAMPPRocky/tokei)

# Installation
Install Rust and then run:
```shell
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

# Command line syntax:
```
Crate or workspace releasing tool. All crates from workspace will be released on crates.io

Usage: releaser [COMMAND]

Commands:
  w           Release workspace specified by path
  c           Release single crate specified by path
  b           Create brew package manager Formula (package definition file) to publish it into a tap (MacOS and Linux only)
  s           Create scoop package manager JSON (package definition file) to publish it into bucket (Windows only)
  completion  Generate the autocompletion script for the specified shell
  bugreport   Collect information about the system and the environment that users can send along with a bug report
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```
Releasing workspace
```
Release workspace specified by path

Usage: releaser w [OPTIONS] <INCR> <PATH>

Arguments:
  <INCR>  Version increment. One of the following: major, minor or patch [possible values: major, minor, patch]
  <PATH>  Sets workspace root path

Options:
  -d, --delay <NUMBER>  Delay in seconds between publish next workflow's crate [default: 20]
  -a, --all             Whether to add option --all-features to cargo publish command
  -n, --noverify        Whether to add option --no-verify to cargo publish command
      --nopublish       Dont publish crate. Just change version, commit, add tag and push changes
  -h, --help            Print help information
```
Releasing simple crate
```
Release single crate specified by path

Usage: releaser c [OPTIONS] <INCR> <PATH>

Arguments:
  <INCR>  Version increment. One of the following: major, minor or patch [possible values: major, minor, patch]
  <PATH>  Sets crate's root path

Options:
  -a, --all       Whether to add option --all-features to cargo publish command
  -n, --noverify  Whether to add option --no-verify to cargo publish command
      --nopublish Dont publish crate. Just change version, commit, add tag and push changes
  -h, --help      Print help information
```
Creating brew package manager Formula (package definition file) to publish it into a tap (MacOS and Linux
only)
```
Create brew package manager Formula (package definition file) to publish it into a tap (MacOS and Linux only)

Usage: releaser b [OPTIONS] --crate <PATH> --base <URI>

Options:
  -c, --crate <PATH>     Sets crate's path where Cargo.toml located
  -l, --linux <PATH>     Sets Linux package directory path
  -m, --macos <PATH>     Sets Mac OS x64-86 package directory path
  -a, --macosarm <PATH>  Sets Mac OS ARM64 package directory path
  -b, --base <URI>       Base URI of downloaded artifacts
  -u, --output [<PATH>]  File path to save result to. If not set result will be written into stdout
  -h, --help             Print help
```
Creating scoop package manager JSON definition file to publish it into a bucket (Windows only)
```
Create scoop package manager JSON (package definition file) to publish it into bucket (Windows only)

Usage: releaser s [OPTIONS] --crate <PATH> --binary <PATH> --exe <FILE> --base <URI>

Options:
  -c, --crate <PATH>     Sets crate's path where Cargo.toml located
  -i, --binary <PATH>    Sets 64-bit binary package directory path
  -e, --exe <FILE>       Sets Windows executable name
  -b, --base <URI>       Base URI of downloaded artifacts
  -u, --output [<PATH>]  File path to save result to. If not set result will be written into stdout
  -h, --help             Print help information
```