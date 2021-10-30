#[macro_use]
extern crate clap;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use std::option::Option::Some;

use releaser::brew;
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
    let crate_path = cmd.value_of("crate").unwrap_or("");
    let linux_path = cmd.value_of("linux").unwrap_or("");
    let macos_path = cmd.value_of("macos").unwrap_or("");

    if linux_path.is_empty() && macos_path.is_empty() {
        return;
    }

    let base_uri = cmd.value_of("base").unwrap_or("");
    let b = brew::new_brew(crate_path, linux_path, macos_path, base_uri);
    if let Some(b) = b {
        println!("{}", b);
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

    match release.release(path, inc.unwrap()) {
        Ok(()) => {}
        Err(e) => eprintln!("Error: {}", e),
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
                .about("Publish brew into tap")
                .arg(
                    Arg::with_name("crate")
                        .long("crate")
                        .short("c")
                        .takes_value(true)
                        .help("Sets crate's path to publish")
                        .required(true),
                )
                .arg(
                    Arg::with_name("linux")
                        .long("linux")
                        .short("l")
                        .takes_value(true)
                        .help("Sets linux package path")
                        .required(false),
                )
                .arg(
                    Arg::with_name("macos")
                        .long("macos")
                        .short("m")
                        .takes_value(true)
                        .help("Sets Mac OS package path")
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
                        .help("File path to save result to. If not set result wiil be written into stdout")
                        .required(false),
                ),
        );
}
