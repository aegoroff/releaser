#[macro_use]
extern crate clap;

use clap::{command, ArgMatches, Command, ArgAction};
use clap_complete::{generate, Shell};
use std::option::Option::Some;
use std::path::PathBuf;
use vfs::{PhysicalFS, VfsPath};
use std::io;

use releaser::brew;
use releaser::cargo::Cargo;
use releaser::git::Git;
use releaser::scoop;
use releaser::workflow::{Crate, Release, VPath, Workspace};
use releaser::Increment;

const PATH: &str = "PATH";
const INCR: &str = "INCR";
const INCR_HELP: &str = "Version increment. One of the following: major, minor or patch";
const ALL: &str = "all";
const NO_VERIFY: &str = "noverify";
const ALL_HELP: &str = "Whether to add option --all-features to cargo publish command";
const NO_VERIFY_HELP: &str = "Whether to add option --no-verify to cargo publish command";
const OUTPUT: &str = "output";
const OUTPUT_HELP: &str =
    "File path to save result to. If not set result will be written into stdout";
const BASE: &str = "base";
const CRATE: &str = "crate";
const BASE_HELP: &str = "Base URI of downloaded artifacts";

fn main() {
    let app = build_cli();
    let matches = app.get_matches();

    match matches.subcommand() {
        Some(("w", cmd)) => workspace(cmd),
        Some(("c", cmd)) => single_crate(cmd),
        Some(("b", cmd)) => brew(cmd),
        Some(("s", cmd)) => scoop(cmd),
        Some(("completion", cmd)) => print_completions(cmd),
        _ => {}
    }
}

fn print_completions(matches: &ArgMatches) {
    let mut cmd = build_cli();
    let bin_name = cmd.get_name().to_string();
    if let Some(generator) = matches.get_one::<Shell>("generator") {
        generate(*generator, &mut cmd, bin_name, &mut io::stdout());
    }
}

fn workspace(cmd: &ArgMatches) {
    let delay_seconds = cmd.get_one::<u64>("delay").unwrap_or(&20);
    let r = Workspace::new(*delay_seconds, Cargo::default(), Git::default());
    release(cmd, &r);
}

fn single_crate(cmd: &ArgMatches) {
    let r = Crate::new(Cargo::default(), Git::default());
    release(cmd, &r);
}

fn brew(cmd: &ArgMatches) {
    let empty = String::default();
    let linux_path = cmd.get_one::<String>("linux").unwrap_or(&empty);
    let macos_path = cmd.get_one::<String>("macos").unwrap_or(&empty);

    if linux_path.is_empty() && macos_path.is_empty() {
        return;
    }

    let crate_path = cmd.get_one::<String>(CRATE).unwrap_or(&empty);
    let base_uri = cmd.get_one::<String>(BASE).unwrap_or(&empty);

    let crate_path: VfsPath = PhysicalFS::new(PathBuf::from(crate_path)).into();
    let linux_path: VfsPath = PhysicalFS::new(PathBuf::from(linux_path)).into();
    let macos_path: VfsPath = PhysicalFS::new(PathBuf::from(macos_path)).into();

    let b = brew::new_brew(&crate_path, &linux_path, &macos_path, base_uri);
    output_string(cmd, b);
}

fn scoop(cmd: &ArgMatches) {
    let empty = String::default();
    let exe_name = cmd.get_one::<String>("exe").unwrap_or(&empty);
    let binary_path = cmd.get_one::<String>("binary").unwrap_or(&empty);
    let crate_path = cmd.get_one::<String>(CRATE).unwrap_or(&empty);
    let base_uri = cmd.get_one::<String>(BASE).unwrap_or(&empty);

    let crate_path: VfsPath = PhysicalFS::new(PathBuf::from(crate_path)).into();
    let binary_path: VfsPath = PhysicalFS::new(PathBuf::from(binary_path)).into();

    let scoop = scoop::new_scoop(&crate_path, &binary_path, &exe_name, base_uri);
    output_string(cmd, scoop);
}

enum ErrorCode {
    NoOutputProduced = 1,
    FileWriteError = 2,
    ReleaseError = 3,
}

/// Helper function that outputs string specified into
/// console or file that set by command line option
fn output_string(cmd: &ArgMatches, s: Option<String>) {
    if let Some(b) = s {
        let output_path = cmd.get_one::<String>(OUTPUT);
        if let Some(path) = output_path {
            let result = std::fs::write(path, b);
            if let Err(e) = result {
                eprintln!("{e}");
                std::process::exit(ErrorCode::FileWriteError as i32);
            }
        } else {
            println!("{b}");
        }
    } else {
        std::process::exit(ErrorCode::NoOutputProduced as i32)
    }
}

/// Helper function that releases crate or workspace
fn release<'a, R>(cmd: &'a ArgMatches, release: &R)
where
    R: Release<'a>,
{
    let path = cmd.get_one::<String>(PATH).unwrap();
    let incr = cmd.get_one::<Increment>(INCR);
    let all_features = cmd.get_flag(ALL);
    let no_verify = cmd.get_flag(NO_VERIFY);

    if incr.is_none() {
        return;
    }

    let r: VfsPath = PhysicalFS::new(PathBuf::from(path)).into();
    let root = VPath::new(path, r);
    if let Err(e) = release.release(root, *incr.unwrap(), all_features, no_verify) {
        eprintln!("Path:\t{path}\nError:\t{e}");
        std::process::exit(ErrorCode::ReleaseError as i32);
    }
}

fn build_cli() -> Command {
    command!(crate_name!())
        .arg_required_else_help(true)
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .subcommand(
            Command::new("w")
                .aliases(["workspace"])
                .about("Release workspace specified by path")
                .arg(
                    arg!([INCR])
                        .value_parser(value_parser!(Increment))
                        .help(INCR_HELP)
                        .required(true)
                        .index(1),
                )
                .arg(
                    arg!([PATH])
                        .help("Sets workspace root path")
                        .required(true)
                        .index(2),
                )
                .arg(
                    arg!(-d --delay <NUMBER>)
                        .required(false)
                        .value_parser(value_parser!(u64))
                        .default_value("20")
                        .help("Delay in seconds between publish next workflow's crate"),
                )
                .arg(
                    arg!(-a --all)
                        .required(false)
                        .action(ArgAction::SetTrue)
                        .help(ALL_HELP),
                )
                .arg(
                    arg!(-n --noverify)
                        .required(false)
                        .action(ArgAction::SetTrue)
                        .help(NO_VERIFY_HELP),
                ),
        )
        .subcommand(
            Command::new("c")
                .aliases(["crate"])
                .about("Release single crate specified by path")
                .arg(
                    arg!([INCR])
                        .value_parser(value_parser!(Increment))
                        .help(INCR_HELP)
                        .required(true)
                        .index(1),
                )
                .arg(
                    arg!([PATH])
                        .help("Sets crate's root path")
                        .required(true)
                        .index(2),
                )
                .arg(
                    arg!(-a --all)
                        .required(false)
                        .action(ArgAction::SetTrue)
                        .help(ALL_HELP),
                )
                .arg(
                    arg!(-n --noverify)
                        .required(false)
                        .action(ArgAction::SetTrue)
                        .help(NO_VERIFY_HELP),
                ),
        )
        .subcommand(
            Command::new("b")
                .aliases(["brew"])
                .about("Create brew package manager Formula (package definition file) to publish it into a tap (MacOS and Linux only)")
                .arg(
                    arg!(-c --crate <PATH>)
                        .required(true)
                        .help("Sets crate's path where Cargo.toml located"),
                )
                .arg(
                    arg!(-l --linux <PATH>)
                        .required(false)
                        .help("Sets Linux package directory path"),
                )
                .arg(
                    arg!(-m --macos <PATH>)
                        .required(false)
                        .help("Sets Mac OS package directory path"),
                )
                .arg(
                    arg!(-b --base <URI>)
                        .required(true)
                        .help(BASE_HELP),
                )
                .arg(
                    arg!(-u --output [PATH])
                        .required(false)
                        .help(OUTPUT_HELP),
                ),
        )
        .subcommand(
            Command::new("s")
                .aliases(["scoop"])
                .about("Create scoop package manager JSON (package definition file) to publish it into bucket (Windows only)")
                .arg(
                    arg!(-c --crate <PATH>)
                        .required(true)
                        .help("Sets crate's path where Cargo.toml located"),
                )
                .arg(
                    arg!(-i --binary <PATH>)
                        .required(true)
                        .help("Sets 64-bit binary package directory path"),
                )
                .arg(
                    arg!(-e --exe <FILE>)
                        .required(true)
                        .help("Sets Windows executable name"),
                )
                .arg(
                    arg!(-b --base <URI>)
                        .required(true)
                        .help(BASE_HELP),
                )
                .arg(
                    arg!(-u --output [PATH])
                        .required(false)
                        .help(OUTPUT_HELP),
                ),
        ).subcommand(
            Command::new("completion")
            .about("Generate the autocompletion script for the specified shell")
            .arg(
                arg!([generator])
                    .value_parser(value_parser!(Shell))
                    .required(true)
                    .index(1),
            )
        )
}
