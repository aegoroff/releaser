# Releaser

[![crates.io](https://img.shields.io/crates/v/releaser.svg)](https://crates.io/crates/releaser)
[![Rust](https://github.com/aegoroff/releaser/actions/workflows/rust.yml/badge.svg)](https://github.com/aegoroff/releaser/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/aegoroff/releaser/branch/master/graph/badge.svg?token=A2vtLxosWU)](https://codecov.io/gh/aegoroff/releaser)
[![](https://tokei.rs/b1/github/aegoroff/releaser?category=code)](https://github.com/XAMPPRocky/tokei)
[![Minimum Stable Rust Version](https://img.shields.io/badge/Rust-1.85.1-blue?color=fc8d62&logo=rust)](https://blog.rust-lang.org/2025/03/18/Rust-1.85.1/)

A powerful command-line tool for automating the release process of Rust crates and workspaces on crates.io, with support for package manager formula generation.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
  - [Releasing a Workspace](#releasing-a-workspace)
  - [Releasing a Single Crate](#releasing-a-single-crate)
  - [Package Manager Formula Generation](#package-manager-formula-generation)
- [Command Reference](#command-reference)
- [Examples](#examples)
- [Contributing](#contributing)
- [License](#license)

## Features

- 🚀 **Automated Version Management**: Automatically increment versions for crates and workspaces
- 🌐 **Workspace Support**: Handle complex workspace releases with dependency management
- 📦 **Package Manager Integration**: Generate Homebrew formulas and Scoop manifests
- 🔧 **Flexible Publishing Options**: Control publishing with various flags and options
- ⏱️ **Smart Delay Handling**: Built-in delays between crate publications to ensure proper indexing
- 🛠️ **Non-publish Mode**: Change crate(s) verisions, commit and git without actually publishing to crates.io
- 📝 **Automatic Git Integration**: Commit changes, create tags, and push to remote repositories
- 🎨 **Shell Completions**: Generate autocompletion scripts for your shell

## Installation

### From crates.io (Recommended)

```bash
cargo install releaser
```

### From Source

```bash
git clone https://github.com/aegoroff/releaser.git
cd releaser
cargo install --path .
```

## Usage

Releaser provides several subcommands for different release scenarios:

### Releasing a Workspace

Release all crates in a workspace with automatic version increment and dependency management:

```bash
# Release a workspace with minor version increment
releaser w minor /path/to/workspace

# Release with a 30-second delay between crate publications
releaser w patch /path/to/workspace -d 30

# Release without actually publishing to crates.io (dry run)
releaser w major /path/to/workspace --nopublish
```

### Releasing a Single Crate

Release a single crate with version increment:

```bash
# Release a single crate with patch version increment
releaser c patch /path/to/crate

# Release with all features enabled
releaser c minor /path/to/crate -a

# Release without verification
releaser c major /path/to/crate -n
```

### Package Manager Formula Generation

Generate package manager formulas for distributing your binaries:

#### Homebrew Formula (macOS/Linux)

```bash
# Generate a Homebrew formula
releaser b \
  --crate /path/to/crate \
  --linux /path/to/linux/binary \
  --macos /path/to/macos/binary \
  --base https://github.com/user/repo/releases/download/vX.Y.Z
```

#### Scoop Manifest (Windows)

```bash
# Generate a Scoop manifest
releaser s \
  --crate /path/to/crate \
  --binary /path/to/windows/binary \
  --exe myapp.exe \
  --base https://github.com/user/repo/releases/download/vX.Y.Z
```

## Command Reference

### `releaser w` - Release Workspace

Release all crates in a workspace.

```bash
releaser w [OPTIONS] <INCR> <PATH>
```

**Arguments:**
- `<INCR>`: Version increment. One of: `major`, `minor`, or `patch`
- `<PATH>`: Path to the workspace root

**Options:**
- `-d, --delay <NUMBER>`: Delay in seconds between publishing crates (default: 20)
- `-a, --all`: Enable all features when publishing
- `-n, --noverify`: Skip verification when publishing
- `--nopublish`: Skip publishing, only update versions and Git operations

### `releaser c` - Release Crate

Release a single crate.

```bash
releaser c [OPTIONS] <INCR> <PATH>
```

**Arguments:**
- `<INCR>`: Version increment. One of: `major`, `minor`, or `patch`
- `<PATH>`: Path to the crate root

**Options:**
- `-a, --all`: Enable all features when publishing
- `-n, --noverify`: Skip verification when publishing
- `--nopublish`: Skip publishing, only update versions and Git operations

### `releaser b` - Generate Homebrew Formula

Create a Homebrew formula for macOS and Linux packages.

```bash
releaser b [OPTIONS] --crate <PATH> --base <URI>
```

**Options:**
- `-c, --crate <PATH>`: Path to the crate where Cargo.toml is located
- `-l, --linux <PATH>`: Path to the Linux package directory
- `-m, --macos <PATH>`: Path to the macOS x64 package directory
- `-a, --macosarm <PATH>`: Path to the macOS ARM64 package directory
- `-b, --base <URI>`: Base URI for downloaded artifacts
- `-u, --output [<PATH>]`: File path to save result (stdout if not set)

### `releaser s` - Generate Scoop Manifest

Create a Scoop manifest for Windows packages.

```bash
releaser s [OPTIONS] --crate <PATH> --binary <PATH> --exe <FILE> --base <URI>
```

**Options:**
- `-c, --crate <PATH>`: Path to the crate where Cargo.toml is located
- `-i, --binary <PATH>`: Path to the 64-bit binary package directory
- `-e, --exe <FILE>`: Windows executable name
- `-b, --base <URI>`: Base URI for downloaded artifacts
- `-u, --output [<PATH>]`: File path to save result (stdout if not set)

### `releaser completion` - Generate Shell Completions

Generate autocompletion scripts for your shell.

```bash
releaser completion <SHELL>
```

Supported shells: bash, elvish, fish, powershell, zsh

### `releaser bugreport` - Generate Bug Report

Collect system information for bug reports.

```bash
releaser bugreport
```

## Examples

### Workspace Release Workflow

```bash
# 1. Release a workspace with minor version increment
releaser w minor /path/to/my/workspace

# 2. Release with custom delay and all features
releaser w patch /path/to/my/workspace -d 60 -a

# 3. Test release without publishing
releaser w major /path/to/my/workspace --nopublish
```

### Single Crate Release

```bash
# Release a single crate with patch increment
releaser c patch /path/to/my/crate

# Release with all features and no verification
releaser c minor /path/to/my/crate -a -n
```

### Package Manager Integration

```bash
# Generate Homebrew formula
releaser b \
  --crate ./my-app \
  --linux ./artifacts/linux \
  --macos ./artifacts/macos \
  --base https://github.com/user/my-app/releases/download/v1.0.0 \
  --output my-app.rb

# Generate Scoop manifest
releaser s \
  --crate ./my-app \
  --binary ./artifacts/windows \
  --exe my-app.exe \
  --base https://github.com/user/my-app/releases/download/v1.0.0 \
  --output my-app.json
```

## Contributing

Contributions are welcome! Here's how you can help:

1. Fork the repository
2. Create a new branch for your feature or bug fix
3. Make your changes and write tests
4. Ensure all tests pass with `cargo test`
5. Submit a pull request with a clear description of your changes

### Development Setup

```bash
# Clone the repository
git clone https://github.com/aegoroff/releaser.git
cd releaser

# Run tests
cargo test

# Build the project
cargo build

# Install from source
cargo install --path .
```

### Code Style

This project uses `rustfmt` for code formatting. Please ensure your code is properly formatted:

```bash
cargo fmt
```

## License

This project is licensed under the MIT License - see the [LICENSE.txt](LICENSE.txt) file for details.

Copyright (c) 2021-2026 Alexander Egorov

## Related Projects

- [cargo-release](https://github.com/sunng87/cargo-release) - Another cargo subcommand for package release
- [cargo-workspaces](https://github.com/pksunkara/cargo-workspaces) - Cargo subcommand for managing workspace versions