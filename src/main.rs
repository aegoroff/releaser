#[macro_use]
extern crate clap;

use bugreport::{
    bugreport,
    collector::{CompileTimeInformation, EnvironmentVariables, OperatingSystem, SoftwareVersion},
    format::Markdown,
};

use clap::{Arg, ArgAction, ArgMatches, Command, command};
use clap_complete::{Shell, generate};
use color_eyre::eyre::{Result, eyre};
use std::io;
use std::option::Option::Some;
use std::path::PathBuf;
use vfs::{PhysicalFS, VfsPath};

use releaser::brew;
use releaser::cargo::Cargo;
use releaser::git::Git;
use releaser::scoop;
use releaser::workflow::{Crate, Release, VPath, Workspace};
use releaser::{Increment, NonPublisher};

const PATH: &str = "PATH";
const FILE: &str = "FILE";
const URI: &str = "URI";
const NUMBER: &str = "NUMBER";
const INCR: &str = "INCR";
const INCR_HELP: &str = "Version increment. One of the following: major, minor or patch";
const ALL: &str = "all";
const NO_VERIFY: &str = "noverify";
const ALL_HELP: &str = "Whether to add option --all-features to cargo publish command";
const NO_VERIFY_HELP: &str = "Whether to add option --no-verify to cargo publish command";
const OUTPUT: &str = "output";
const OUTPUT_HELP: &str =
    "File path to save result to. If not set result will be written into stdout";
const NO_PUBLISH: &str = "nopublish";
const NO_PUBLISH_HELP: &str =
    "Dont publish crate. Just change version, commit, add tag and push changes";
const BASE: &str = "base";
const CRATE: &str = "crate";
const BASE_HELP: &str = "Base URI of downloaded artifacts";
const EXE: &str = "exe";
const BINARY: &str = "binary";
const DELAY: &str = "delay";
const LINUX: &str = "linux";
const MACOS: &str = "macos";
const MACOSARM: &str = "macosarm";

fn main() -> Result<()> {
    color_eyre::install()?;
    let app = build_cli();
    let matches = app.get_matches();

    match matches.subcommand() {
        Some(("w", cmd)) => workspace(cmd),
        Some(("c", cmd)) => single_crate(cmd),
        Some(("b", cmd)) => brew(cmd),
        Some(("s", cmd)) => scoop(cmd),
        Some(("completion", cmd)) => {
            print_completions(cmd);
            Ok(())
        }
        Some(("bugreport", cmd)) => {
            print_bugreport(cmd);
            Ok(())
        }
        _ => Ok(()),
    }
}

fn print_completions(matches: &ArgMatches) {
    let mut cmd = build_cli();
    let bin_name = cmd.get_name().to_string();
    if let Some(generator) = matches.get_one::<Shell>("generator") {
        generate(*generator, &mut cmd, bin_name, &mut io::stdout());
    }
}

fn print_bugreport(_matches: &ArgMatches) {
    bugreport!()
        .info(SoftwareVersion::default())
        .info(OperatingSystem::default())
        .info(EnvironmentVariables::list(&["SHELL", "TERM"]))
        .info(CompileTimeInformation::default())
        .print::<Markdown>();
}

fn workspace(cmd: &ArgMatches) -> Result<()> {
    let delay_seconds = cmd.get_one::<u64>(DELAY).unwrap_or(&20);
    if cmd.get_flag(NO_PUBLISH) {
        let r = Workspace::new(*delay_seconds, NonPublisher, Git);
        release(cmd, &r)
    } else {
        let r = Workspace::new(*delay_seconds, Cargo, Git);
        release(cmd, &r)
    }
}

fn single_crate(cmd: &ArgMatches) -> Result<()> {
    if cmd.get_flag(NO_PUBLISH) {
        let r = Crate::new(NonPublisher, Git);
        release(cmd, &r)
    } else {
        let r = Crate::new(Cargo, Git);
        release(cmd, &r)
    }
}

fn brew(cmd: &ArgMatches) -> Result<()> {
    let empty = String::default();
    let linux_path = cmd.get_one::<String>(LINUX).unwrap_or(&empty);
    let macos_path = cmd.get_one::<String>(MACOS).unwrap_or(&empty);
    let macos_arm_path = cmd.get_one::<String>(MACOSARM).unwrap_or(&empty);

    if linux_path.is_empty() && macos_path.is_empty() {
        return Ok(());
    }

    let crate_path = cmd.get_one::<String>(CRATE).unwrap_or(&empty);
    let base_uri = cmd.get_one::<String>(BASE).unwrap_or(&empty);

    let crate_path: VfsPath = PhysicalFS::new(PathBuf::from(crate_path)).into();
    let linux_path: VfsPath = PhysicalFS::new(PathBuf::from(linux_path)).into();
    let macos_path: VfsPath = PhysicalFS::new(PathBuf::from(macos_path)).into();
    let macos_arm_path: VfsPath = PhysicalFS::new(PathBuf::from(macos_arm_path)).into();
    let b = brew::Brew::serialize(
        &crate_path,
        &linux_path,
        &macos_path,
        &macos_arm_path,
        base_uri,
    )?;
    output_string(cmd, b)
}

fn scoop(cmd: &ArgMatches) -> Result<()> {
    let empty = String::default();
    let exe_name = cmd.get_one::<String>(EXE).unwrap_or(&empty);
    let binary_path = cmd.get_one::<String>(BINARY).unwrap_or(&empty);
    let crate_path = cmd.get_one::<String>(CRATE).unwrap_or(&empty);
    let base_uri = cmd.get_one::<String>(BASE).unwrap_or(&empty);

    let crate_path: VfsPath = PhysicalFS::new(PathBuf::from(crate_path)).into();
    let binary_path: VfsPath = PhysicalFS::new(PathBuf::from(binary_path)).into();

    let scoop = scoop::Scoop::serialize(&crate_path, &binary_path, exe_name, base_uri)?;
    output_string(cmd, scoop)
}

/// Helper function that outputs string specified into
/// console or file that set by command line option
fn output_string(cmd: &ArgMatches, s: String) -> Result<()> {
    if s.is_empty() {
        Err(eyre!("No output produced but it should"))
    } else {
        let output_path = cmd.get_one::<String>(OUTPUT);
        if let Some(path) = output_path {
            std::fs::write(path, s)?;
        } else {
            println!("{s}");
        }
        Ok(())
    }
}

/// Helper function that releases crate or workspace
fn release<'a, R>(cmd: &'a ArgMatches, release: &R) -> Result<()>
where
    R: Release<'a>,
{
    let path = cmd.get_one::<String>(PATH).unwrap();
    let incr = cmd.get_one::<Increment>(INCR);
    let all_features = cmd.get_flag(ALL);
    let no_verify = cmd.get_flag(NO_VERIFY);

    if incr.is_none() {
        return Ok(());
    }

    let r: VfsPath = PhysicalFS::new(PathBuf::from(path)).into();
    let root = VPath::new(path, r);
    release.release(root, *incr.unwrap(), all_features, no_verify)
}

fn build_cli() -> Command {
    #![allow(non_upper_case_globals)]
    command!(crate_name!())
        .arg_required_else_help(true)
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .subcommand(workspace_cmd())
        .subcommand(crate_cmd())
        .subcommand(brew_cmd())
        .subcommand(scoop_cmd())
        .subcommand(completion_cmd())
        .subcommand(bugreport_cmd())
}

fn workspace_cmd() -> Command {
    Command::new("w")
        .aliases(["workspace"])
        .about("Release workspace specified by path")
        .arg(increment_arg())
        .arg(
            Arg::new(PATH)
                .help("Sets workspace root path")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::new(DELAY)
                .long(DELAY)
                .short('d')
                .value_name(NUMBER)
                .required(false)
                .value_parser(value_parser!(u64))
                .default_value("20")
                .help("Delay in seconds between publish next workflow's crate"),
        )
        .arg(all_arg())
        .arg(noverify_arg())
        .arg(nopublish_arg())
}

fn crate_cmd() -> Command {
    Command::new("c")
        .aliases(["crate"])
        .about("Release single crate specified by path")
        .arg(increment_arg())
        .arg(
            Arg::new(PATH)
                .help("Sets crate's root path")
                .required(true)
                .index(2),
        )
        .arg(all_arg())
        .arg(noverify_arg())
        .arg(nopublish_arg())
}

fn brew_cmd() -> Command {
    Command::new("b")
        .aliases(["brew"])
        .about("Create brew package manager Formula (package definition file) to publish it into a tap (MacOS and Linux only)")
        .arg(crate_arg())
        .arg(
            Arg::new(LINUX)
                .long(LINUX)
                .short('l')
                .value_name(PATH)
                .required(false)
                .help("Sets Linux package directory path"),
        )
        .arg(
            Arg::new(MACOS)
                .long(MACOS)
                .short('m')
                .value_name(PATH)
                .required(false)
                .help("Sets Mac OS x64-86 package directory path"),
        )
        .arg(
            Arg::new(MACOSARM)
                .long(MACOSARM)
                .short('a')
                .value_name(PATH)
                .required(false)
                .help("Sets Mac OS ARM64 package directory path"),
        )
        .arg(base_arg())
        .arg(output_arg())
}

fn scoop_cmd() -> Command {
    Command::new("s")
        .aliases(["scoop"])
        .about("Create scoop package manager JSON (package definition file) to publish it into bucket (Windows only)")
        .arg(crate_arg())
        .arg(
            Arg::new(BINARY)
                .long(BINARY)
                .short('i')
                .value_name(PATH)
                .required(true)
                .help("Sets 64-bit binary package directory path"),
        )
        .arg(
            Arg::new(EXE)
                .long(EXE)
                .short('e')
                .value_name(FILE)
                .required(true)
                .help("Sets Windows executable name"),
        )
        .arg(base_arg())
        .arg(output_arg())
}

fn completion_cmd() -> Command {
    Command::new("completion")
        .about("Generate the autocompletion script for the specified shell")
        .arg(
            arg!([generator])
                .value_parser(value_parser!(Shell))
                .required(true)
                .index(1),
        )
}

fn bugreport_cmd() -> Command {
    Command::new("bugreport")
        .about("Collect information about the system and the environment that users can send along with a bug report")
}

fn increment_arg() -> Arg {
    Arg::new(INCR)
        .value_parser(value_parser!(Increment))
        .help(INCR_HELP)
        .required(true)
        .index(1)
}

fn base_arg() -> Arg {
    Arg::new(BASE)
        .long(BASE)
        .short('b')
        .value_name(URI)
        .required(true)
        .help(BASE_HELP)
}

fn output_arg() -> Arg {
    Arg::new(OUTPUT)
        .long(OUTPUT)
        .short('u')
        .num_args(0..=1)
        .value_name(PATH)
        .required(false)
        .help(OUTPUT_HELP)
}

fn crate_arg() -> Arg {
    Arg::new(CRATE)
        .long(CRATE)
        .short('c')
        .value_name(PATH)
        .required(true)
        .help("Sets crate's path where Cargo.toml located")
}

fn noverify_arg() -> Arg {
    Arg::new(NO_VERIFY)
        .long(NO_VERIFY)
        .short('n')
        .required(false)
        .action(ArgAction::SetTrue)
        .help(NO_VERIFY_HELP)
}

fn nopublish_arg() -> Arg {
    Arg::new(NO_PUBLISH)
        .long(NO_PUBLISH)
        .required(false)
        .action(ArgAction::SetTrue)
        .help(NO_PUBLISH_HELP)
}

fn all_arg() -> Arg {
    Arg::new(ALL)
        .long(ALL)
        .short('a')
        .required(false)
        .action(ArgAction::SetTrue)
        .help(ALL_HELP)
}
