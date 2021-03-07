use clap::{App, Arg, SubCommand};
use releaser::workflow;
use releaser::Increment;

extern crate clap;

fn main() {
    let app = build_cli();
    let matches = app.get_matches();

    if let Some(cmd) = matches.subcommand_matches("p") {
        if let Some(path) = cmd.value_of("PATH") {
            match workflow::release(path, Increment::Patch) {
                Ok(()) => {}
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    }

    if let Some(cmd) = matches.subcommand_matches("mi") {
        if let Some(path) = cmd.value_of("PATH") {
            match workflow::release(path, Increment::Minor) {
                Ok(()) => {}
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    }

    if let Some(cmd) = matches.subcommand_matches("ma") {
        if let Some(path) = cmd.value_of("PATH") {
            match workflow::release(path, Increment::Major) {
                Ok(()) => {}
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    }
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
