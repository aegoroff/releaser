use clap::{App, Arg, ArgMatches, SubCommand};
use releaser::workflow;
use releaser::Increment;

#[macro_use]
extern crate clap;

const PATH: &str = "PATH";
const INCR: &str = "INCR";

fn main() {
    let app = build_cli();
    let matches = app.get_matches();

    if let Some(cmd) = matches.subcommand_matches("w") {
        release(cmd, workflow::release_workspace);
    }

    if let Some(cmd) = matches.subcommand_matches("c") {
        release(cmd, workflow::release_crate);
    }
}

fn release<F>(cmd: &ArgMatches, action: F)
where
    F: Fn(&str, Increment) -> releaser::Result<()>,
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

    match action(path, inc.unwrap()) {
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
