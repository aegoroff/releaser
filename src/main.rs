use clap::{App, Arg, SubCommand};
use releaser::workflow;
use releaser::Increment;

extern crate clap;

macro_rules! command {
    ($m:ident, $cmd:expr, $inc:ident) => {
        if let Some(cmd) = $m.subcommand_matches($cmd) {
            if let Some(path) = cmd.value_of("PATH") {
                match workflow::release(path, Increment::$inc) {
                    Ok(()) => {}
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        }
    };
}

fn main() {
    let app = build_cli();
    let matches = app.get_matches();

    command!(matches, "p", Patch);
    command!(matches, "mi", Minor);
    command!(matches, "ma", Major);
}

fn build_cli() -> App<'static, 'static> {
    return App::new("releaser")
        .version("0.1")
        .author("egoroff <egoroff@gmail.com>")
        .about("Rust releasing workspace tool")
        .subcommand(
            SubCommand::with_name("p")
                .aliases(&["patch"])
                .about("Release patch version")
                .arg(
                    Arg::with_name("PATH")
                        .help("Sets workspace root path")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("mi")
                .aliases(&["minor"])
                .about("Release minor version")
                .arg(
                    Arg::with_name("PATH")
                        .help("Sets workspace root path")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("ma")
                .aliases(&["major"])
                .about("Release major version")
                .arg(
                    Arg::with_name("PATH")
                        .help("Sets workspace root path")
                        .required(true)
                        .index(1),
                ),
        );
}
