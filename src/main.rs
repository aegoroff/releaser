#[macro_use]
extern crate clap;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use std::option::Option::Some;
use std::path::PathBuf;
use vfs::{PhysicalFS, VfsPath};

use releaser::brew;
use releaser::scoop;
use releaser::workflow::{Crate, Release, Workspace};
use releaser::Increment;

const PATH: &str = "PATH";
const INCR: &str = "INCR";

fn main() {
    let app = build_cli();
    let matches = app.get_matches();

    match matches.subcommand() {
        ("w", Some(cmd)) => workspace(cmd),
        ("c", Some(cmd)) => single_crate(cmd),
        ("b", Some(cmd)) => brew(cmd),
        ("s", Some(cmd)) => scoop(cmd),
        _ => {}
    }
}

fn workspace(cmd: &ArgMatches) {
    let delay_seconds = cmd.value_of("delay").unwrap_or("");
    let delay_seconds: u64 = delay_seconds.parse().unwrap_or(20);
    let r = Workspace::new(delay_seconds);
    release(cmd, r);
}

fn single_crate(cmd: &ArgMatches) {
    let r = Crate::new();
    release(cmd, r);
}

fn brew(cmd: &ArgMatches) {
    let linux_path = cmd.value_of("linux").unwrap_or("");
    let macos_path = cmd.value_of("macos").unwrap_or("");

    if linux_path.is_empty() && macos_path.is_empty() {
        return;
    }

    let crate_path = cmd.value_of("crate").unwrap_or("");
    let base_uri = cmd.value_of("base").unwrap_or("");
    let b = brew::new_brew(crate_path, linux_path, macos_path, base_uri);
    output_string(cmd, b)
}

fn scoop(cmd: &ArgMatches) {
    let exe_name = cmd.value_of("exe").unwrap_or("");
    let binary_path = cmd.value_of("binary").unwrap_or("");
    let crate_path = cmd.value_of("crate").unwrap_or("");
    let base_uri = cmd.value_of("base").unwrap_or("");
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
            let output_path = cmd.value_of("output");
            match output_path {
                None => println!("{}", b),
                Some(path) => {
                    let result = std::fs::write(path, b);
                    match result {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("{}", e);
                            std::process::exit(ErrorCode::FileWriteError as i32);
                        }
                    }
                }
            }
        }
    }
}

fn release<R>(cmd: &ArgMatches, release: R)
where
    R: Release,
{
    let path = cmd.value_of(PATH).unwrap();
    let incr = cmd.value_of(INCR).unwrap();

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
    match release.release(r, inc.unwrap()) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(ErrorCode::ReleaseError as i32);
        }
    }
}

fn build_cli() -> App<'static, 'static> {
    return App::new(crate_name!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(crate_version!())
        .author("egoroff <egoroff@gmail.com>")
        .about("Rust releasing workspace tool")
        .subcommand(
            SubCommand::with_name("w")
                .aliases(&["workspace"])
                .about("Release workspace specified by path")
                .arg(
                    Arg::with_name(INCR)
                        .help("Version increment. One of the following: major, minor or patch")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name(PATH)
                        .help("Sets workspace root path")
                        .required(true)
                        .index(2),
                )
                .arg(
                    Arg::with_name("delay")
                        .long("delay")
                        .short("d")
                        .takes_value(true)
                        .default_value("20")
                        .help("Delay in seconds between publish next workflow's crate")
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("c")
                .aliases(&["crate"])
                .about("Release single crate specified by path")
                .arg(
                    Arg::with_name(INCR)
                        .help("Version increment. One of the following: major, minor or patch")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name(PATH)
                        .help("Sets crate's root path")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("b")
                .aliases(&["brew"])
                .about("Create brew package manager Formula (package definition file) to publish it into a tap (MacOS and Linux only)")
                .arg(
                    Arg::with_name("crate")
                        .long("crate")
                        .short("c")
                        .takes_value(true)
                        .help("Sets crate's path where Cargo.toml located")
                        .required(true),
                )
                .arg(
                    Arg::with_name("linux")
                        .long("linux")
                        .short("l")
                        .takes_value(true)
                        .help("Sets Linux package directory path")
                        .required(false),
                )
                .arg(
                    Arg::with_name("macos")
                        .long("macos")
                        .short("m")
                        .takes_value(true)
                        .help("Sets Mac OS package directory path")
                        .required(false),
                )
                .arg(
                    Arg::with_name("base")
                        .long("base")
                        .short("b")
                        .takes_value(true)
                        .help("Base URI of downloaded artifacts")
                        .required(true),
                )
                .arg(
                    Arg::with_name("output")
                        .long("output")
                        .short("u")
                        .takes_value(true)
                        .help("File path to save result to. If not set result will be written into stdout")
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("s")
                .aliases(&["scoop"])
                .about("Create scoop package manager JSON (package definition file) to publish it into bucket (Windows only)")
                .arg(
                    Arg::with_name("crate")
                        .long("crate")
                        .short("c")
                        .takes_value(true)
                        .help("Sets crate's path where Cargo.toml located")
                        .required(true),
                )
                .arg(
                    Arg::with_name("binary")
                        .long("binary")
                        .short("i")
                        .takes_value(true)
                        .help("Sets 64-bit binary package directory path")
                        .required(true),
                )
                .arg(
                    Arg::with_name("exe")
                        .long("exe")
                        .short("e")
                        .takes_value(true)
                        .help("Sets Windows executable name")
                        .required(true),
                )
                .arg(
                    Arg::with_name("base")
                        .long("base")
                        .short("b")
                        .takes_value(true)
                        .help("Base URI of downloaded artifacts")
                        .required(true),
                )
                .arg(
                    Arg::with_name("output")
                        .long("output")
                        .short("u")
                        .takes_value(true)
                        .help("File path to save result to. If not set result will be written into stdout")
                        .required(false),
                ),
        );
}
