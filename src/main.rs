use clap::{App, Arg, ArgMatches, SubCommand};
use releaser::workflow::{Crate, Release, Workspace};
use releaser::Increment;

#[macro_use]
extern crate clap;

const PATH: &str = "PATH";
const INCR: &str = "INCR";

fn main() {
    let app = build_cli();
    let matches = app.get_matches();

    match matches.subcommand() {
        ("w", Some(cmd)) => workspace(cmd),
        ("c", Some(cmd)) => single_crate(cmd),
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
    return App::new("releaser")
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
        );
}
