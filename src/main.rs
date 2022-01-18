#[macro_use]
extern crate clap;

use clap::{App, AppSettings, Arg, ArgMatches};
use std::option::Option::Some;
use std::path::PathBuf;
use vfs::{PhysicalFS, VfsPath};

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
const ALL_HELP: &str = "Whether to add option --all-features to cargo publish command";
const OUTPUT: &str = "output";
const OUTPUT_HELP: &str =
    "File path to save result to. If not set result will be written into stdout";
const BASE: &str = "base";
const BASE_HELP: &str = "Base URI of downloaded artifacts";

fn main() {
    let app = build_cli();
    let matches = app.get_matches();

    match matches.subcommand() {
        Some(("w", cmd)) => workspace(cmd),
        Some(("c", cmd)) => single_crate(cmd),
        Some(("b", cmd)) => brew(cmd),
        Some(("s", cmd)) => scoop(cmd),
        _ => {}
    }
}

fn workspace(cmd: &ArgMatches) {
    let delay_seconds = cmd.value_of("delay").unwrap_or("");
    let delay_seconds: u64 = delay_seconds.parse().unwrap_or(20);
    let r = Workspace::new(delay_seconds, Cargo::default(), Git::default());
    release(cmd, r);
}

fn single_crate(cmd: &ArgMatches) {
    let r = Crate::new(Cargo::default(), Git::default());
    release(cmd, r);
}

fn brew(cmd: &ArgMatches) {
    let linux_path = cmd.value_of("linux").unwrap_or("");
    let macos_path = cmd.value_of("macos").unwrap_or("");

    if linux_path.is_empty() && macos_path.is_empty() {
        return;
    }

    let crate_path = cmd.value_of("crate").unwrap_or("");
    let base_uri = cmd.value_of(BASE).unwrap_or("");

    let crate_path: VfsPath = PhysicalFS::new(PathBuf::from(crate_path)).into();
    let linux_path: VfsPath = PhysicalFS::new(PathBuf::from(linux_path)).into();
    let macos_path: VfsPath = PhysicalFS::new(PathBuf::from(macos_path)).into();

    let b = brew::new_brew(crate_path, linux_path, macos_path, base_uri);
    output_string(cmd, b)
}

fn scoop(cmd: &ArgMatches) {
    let exe_name = cmd.value_of("exe").unwrap_or("");
    let binary_path = cmd.value_of("binary").unwrap_or("");
    let crate_path = cmd.value_of("crate").unwrap_or("");
    let base_uri = cmd.value_of(BASE).unwrap_or("");

    let crate_path: VfsPath = PhysicalFS::new(PathBuf::from(crate_path)).into();
    let binary_path: VfsPath = PhysicalFS::new(PathBuf::from(binary_path)).into();

    let scoop = scoop::new_scoop(crate_path, binary_path, exe_name, base_uri);
    output_string(cmd, scoop)
}

enum ErrorCode {
    NoOutputProduced = 1,
    FileWriteError = 2,
    ReleaseError = 3,
}

fn output_string(cmd: &ArgMatches, s: Option<String>) {
    match s {
        None => std::process::exit(ErrorCode::NoOutputProduced as i32),
        Some(b) => {
            let output_path = cmd.value_of(OUTPUT);
            match output_path {
                None => println!("{b}"),
                Some(path) => {
                    let result = std::fs::write(path, b);
                    match result {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("{e}");
                            std::process::exit(ErrorCode::FileWriteError as i32);
                        }
                    }
                }
            }
        }
    }
}

fn release<'a, R>(cmd: &'a ArgMatches, release: R)
where
    R: Release<'a>,
{
    let path = cmd.value_of(PATH).unwrap();
    let incr = cmd.value_of(INCR).unwrap();
    let all_features = cmd.is_present(ALL);

    let inc = match incr {
        "major" => Some(Increment::Major),
        "minor" => Some(Increment::Minor),
        "patch" => Some(Increment::Patch),
        _ => None,
    };

    if inc.is_none() {
        return;
    }

    let r: VfsPath = PhysicalFS::new(PathBuf::from(path)).into();
    let root = VPath::new(path, r);
    match release.release(root, inc.unwrap(), all_features) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Path:\t{}\nError:\t{}", path, e);
            std::process::exit(ErrorCode::ReleaseError as i32);
        }
    }
}

fn build_cli() -> App<'static> {
    return App::new(crate_name!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(crate_version!())
        .author("egoroff <egoroff@gmail.com>")
        .about("Rust releasing workspace tool")
        .subcommand(
            App::new("w")
                .aliases(&["workspace"])
                .about("Release workspace specified by path")
                .arg(
                    Arg::new(INCR)
                        .help(INCR_HELP)
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new(PATH)
                        .help("Sets workspace root path")
                        .required(true)
                        .index(2),
                )
                .arg(
                    Arg::new("delay")
                        .long("delay")
                        .short('d')
                        .takes_value(true)
                        .default_value("20")
                        .help("Delay in seconds between publish next workflow's crate")
                        .required(false),
                )
                .arg(
                    Arg::new(ALL)
                        .long(ALL)
                        .short('a')
                        .takes_value(false)
                        .help(ALL_HELP)
                        .required(false),
                ),
        )
        .subcommand(
            App::new("c")
                .aliases(&["crate"])
                .about("Release single crate specified by path")
                .arg(
                    Arg::new(INCR)
                        .help(INCR_HELP)
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new(PATH)
                        .help("Sets crate's root path")
                        .required(true)
                        .index(2),
                )
                .arg(
                    Arg::new(ALL)
                        .long(ALL)
                        .short('a')
                        .takes_value(false)
                        .help(ALL_HELP)
                        .required(false),
                ),
        )
        .subcommand(
            App::new("b")
                .aliases(&["brew"])
                .about("Create brew package manager Formula (package definition file) to publish it into a tap (MacOS and Linux only)")
                .arg(
                    Arg::new("crate")
                        .long("crate")
                        .short('c')
                        .takes_value(true)
                        .help("Sets crate's path where Cargo.toml located")
                        .required(true),
                )
                .arg(
                    Arg::new("linux")
                        .long("linux")
                        .short('l')
                        .takes_value(true)
                        .help("Sets Linux package directory path")
                        .required(false),
                )
                .arg(
                    Arg::new("macos")
                        .long("macos")
                        .short('m')
                        .takes_value(true)
                        .help("Sets Mac OS package directory path")
                        .required(false),
                )
                .arg(
                    Arg::new(BASE)
                        .long(BASE)
                        .short('b')
                        .takes_value(true)
                        .help(BASE_HELP)
                        .required(true),
                )
                .arg(
                    Arg::new(OUTPUT)
                        .long(OUTPUT)
                        .short('u')
                        .takes_value(true)
                        .help(OUTPUT_HELP)
                        .required(false),
                ),
        )
        .subcommand(
            App::new("s")
                .aliases(&["scoop"])
                .about("Create scoop package manager JSON (package definition file) to publish it into bucket (Windows only)")
                .arg(
                    Arg::new("crate")
                        .long("crate")
                        .short('c')
                        .takes_value(true)
                        .help("Sets crate's path where Cargo.toml located")
                        .required(true),
                )
                .arg(
                    Arg::new("binary")
                        .long("binary")
                        .short('i')
                        .takes_value(true)
                        .help("Sets 64-bit binary package directory path")
                        .required(true),
                )
                .arg(
                    Arg::new("exe")
                        .long("exe")
                        .short('e')
                        .takes_value(true)
                        .help("Sets Windows executable name")
                        .required(true),
                )
                .arg(
                    Arg::new(BASE)
                        .long(BASE)
                        .short('b')
                        .takes_value(true)
                        .help(BASE_HELP)
                        .required(true),
                )
                .arg(
                    Arg::new(OUTPUT)
                        .long(OUTPUT)
                        .short('u')
                        .takes_value(true)
                        .help(OUTPUT_HELP)
                        .required(false),
                ),
        );
}
